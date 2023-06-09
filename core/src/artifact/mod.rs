use std::collections::HashMap;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use typst::model::Locator;

use crate::annotation::link::AnnotationProcessor;
use crate::font::get_font_coverage_hash;
use crate::FontResolver;

pub mod doc;
use doc::*;

pub mod font;
use font::*;

pub mod geom;
use geom::*;

pub mod image;
use image::*;

pub mod core;
pub use self::core::BuildInfo;
use self::core::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    /// metadata about this artifact
    #[serde(flatten)]
    pub meta: ArtifactMeta,
    /// The page frames.
    pub pages: Vec<Frame>,
}

pub struct ArtifactBuilder {
    fonts: Vec<TypstFontInfo>,
    font_map: HashMap<TypstFontInfo, FontRef>,
    annoation_proc: AnnotationProcessor,
}

impl ArtifactBuilder {
    pub fn new(typst_doc: &typst::doc::Document) -> Self {
        Self {
            fonts: vec![],
            font_map: HashMap::default(),
            annoation_proc: AnnotationProcessor::new(typst_doc),
        }
    }

    pub fn write_font(&mut self, font: &TypstFont) -> FontRef {
        if let Some(font) = self.font_map.get(font.info()) {
            return *font;
        }

        if self.fonts.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_ref = self.fonts.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.fonts.push(font.info().clone());
        font_ref
    }

    pub fn write_span(&mut self, _span: TypstSpan) -> SpanRef {
        // todo
        0
    }

    pub fn write_glyph(&mut self, glyph: &TypstGlyph) -> Glyph {
        Glyph {
            id: glyph.id,
            x_advance: glyph.x_advance.into(),
            x_offset: glyph.x_offset.into(),
            span: (self.write_span(glyph.span.0), glyph.span.1),
            range: (glyph.range.start, glyph.range.end),
        }
    }

    pub fn write_text_item(&mut self, text: &TypstTextItem) -> TextItem {
        let idx = self.write_font(&text.font);
        TextItem {
            font: idx,
            size: text.size.into(),
            fill: text.fill.clone().into(),
            lang: text.lang.as_str().to_string(),
            text: text.text.as_str().to_string(),
            glyphs: text.glyphs.iter().map(|g| self.write_glyph(g)).collect(),
        }
    }

    pub fn write_group_item(&mut self, group: &TypstGroupItem) -> GroupItem {
        GroupItem {
            frame: self.write_frame(&group.frame),
            transform: group.transform.into(),
            clips: group.clips,
        }
    }

    pub fn write_image(&mut self, image: &TypstImage) -> Image {
        image.clone().into()
    }

    pub fn write_frame_item(&mut self, item: &TypstFrameItem) -> FrameItem {
        match item {
            TypstFrameItem::Group(group) => FrameItem::Group(self.write_group_item(group)),
            TypstFrameItem::Text(text) => FrameItem::Text(self.write_text_item(text)),
            TypstFrameItem::Shape(shape, _) => FrameItem::Shape(shape.clone().into()),
            TypstFrameItem::Image(image, size, _) => {
                FrameItem::Image(self.write_image(image), (*size).into())
            }
            TypstFrameItem::Meta(meta, size) => match meta {
                TypstMeta::Link(dest) => FrameItem::MetaLink(
                    match dest {
                        TypstDestination::Url(url) => Destination::Url(url.as_str().to_string()),
                        TypstDestination::Position(pos) => Destination::Position(Position {
                            page: pos.page,
                            point: pos.point.into(),
                        }),
                        TypstDestination::Location(loc) => {
                            let pos = self.annoation_proc.process_location(*loc);
                            Destination::Position(Position {
                                page: pos.page,
                                point: pos.point.into(),
                            })
                        }
                    },
                    (*size).into(),
                ),
                TypstMeta::Elem(_) => FrameItem::None,
                TypstMeta::Hide => FrameItem::None,
                TypstMeta::PageNumbering(_) => FrameItem::None,
            },
        }
    }

    pub fn write_frame(&mut self, frame: &TypstFrame) -> Frame {
        Frame {
            size: Axes {
                x: frame.width().into(),
                y: frame.height().into(),
            },
            baseline: if frame.has_baseline() {
                Some(frame.baseline().into())
            } else {
                None
            },
            items: frame
                .items()
                .map(|item| (item.0.into(), self.write_frame_item(&item.1)))
                .collect(),
        }
    }
}

impl From<&TypstDocument> for Artifact {
    fn from(typst_doc: &TypstDocument) -> Self {
        let mut builder = ArtifactBuilder::new(typst_doc);

        let pages = typst_doc
            .pages
            .iter()
            .map(|f| builder.write_frame(f))
            .collect();

        let meta = ArtifactMeta {
            build: Some(BuildInfo {
                compiler: "typst-ts-cli".to_string(),
                version: crate::build_info::VERSION.to_string(),
            }),
            fonts: builder
                .fonts
                .into_iter()
                .map(|info| FontInfo {
                    family: info.family,
                    variant: info.variant,
                    flags: info.flags.bits(),
                    coverage: FontCoverage::from_vec(info.coverage.iter().take(1).collect()),
                    coverage_hash: get_font_coverage_hash(&info.coverage),
                })
                .collect(),
            title: typst_doc.title.as_ref().map(|s| s.to_string()),
            author: typst_doc.author.iter().map(|s| s.to_string()).collect(),
        };

        Self { meta, pages }
    }
}

pub struct TypeDocumentParser {
    sp: Locator<'static>,
    fonts: Vec<TypstFont>,
}

impl TypeDocumentParser {
    pub fn new() -> Self {
        Self {
            sp: Locator::new(),
            fonts: Vec::new(),
        }
    }

    pub fn get_font(&mut self, font: FontRef) -> TypstFont {
        if let Some(f) = self.fonts.get(font as usize) {
            return f.clone();
        }
        panic!("Out of bounds font index {}", font);
    }

    pub fn parse_glyph(&mut self, glyph: &Glyph) -> TypstGlyph {
        TypstGlyph {
            id: glyph.id,
            x_advance: glyph.x_advance.into(),
            x_offset: glyph.x_offset.into(),
            span: (TypstSpan::detached(), glyph.span.1),
            range: glyph.range.0..glyph.range.1,
        }
    }

    pub fn parse_group_item(&mut self, group: &GroupItem) -> TypstGroupItem {
        TypstGroupItem {
            frame: self.parse_frame(&group.frame),
            transform: group.transform.into(),
            clips: group.clips,
        }
    }

    pub fn parse_text_item(&mut self, text: &TextItem) -> TypstTextItem {
        TypstTextItem {
            font: self.get_font(text.font),
            size: text.size.into(),
            fill: text.fill.clone().into(),
            lang: TypstLang::from_str(text.lang.as_str()).unwrap(),
            text: text.text.clone().into(),
            glyphs: text.glyphs.iter().map(|g| self.parse_glyph(g)).collect(),
        }
    }

    pub fn parse_image(&mut self, image: &Image) -> TypstImage {
        image.clone().into()
    }

    pub fn parse_location(&mut self, loc: &str) -> Option<TypstLocation> {
        let loc_hash = loc.parse().ok()?;
        Some(self.sp.locate(loc_hash))
    }

    pub fn parse_frame_item(&mut self, item: &FrameItem) -> TypstFrameItem {
        match item {
            FrameItem::Group(group) => TypstFrameItem::Group(self.parse_group_item(group)),
            FrameItem::Text(text) => TypstFrameItem::Text(self.parse_text_item(text)),
            FrameItem::Shape(shape) => {
                TypstFrameItem::Shape(shape.clone().into(), TypstSpan::detached())
            }
            FrameItem::Image(image, size) => TypstFrameItem::Image(
                self.parse_image(image),
                (*size).into(),
                TypstSpan::detached(),
            ),
            FrameItem::MetaLink(dest, size) => {
                let dest = match dest {
                    Destination::Url(url) => {
                        TypstDestination::Url(TypstEcoString::from(url.as_str()))
                    }
                    Destination::Position(pos) => TypstDestination::Position(TypstPosition {
                        page: pos.page,
                        point: pos.point.into(),
                    }),
                    // Destination::Location(loc) => match self.parse_location(loc) {
                    //     Some(loc) => TypstDestination::Location(loc),
                    //     None => panic!("Invalid location: {}", loc),
                    // },
                };

                TypstFrameItem::Meta(TypstMeta::Link(dest), (*size).into())
            }
            FrameItem::None => panic!("None frame item"),
        }
    }

    pub fn parse_frame(&mut self, frame: &Frame) -> TypstFrame {
        let mut parsed_frame = TypstFrame::new(frame.size.into());
        if let Some(bl) = frame.baseline {
            parsed_frame.set_baseline(bl.into())
        }

        for (pos, item) in frame.items.iter() {
            match item {
                FrameItem::None => continue,
                _ => {
                    parsed_frame.push((*pos).into(), self.parse_frame_item(item));
                }
            };
        }

        parsed_frame
    }
}

impl Artifact {
    pub fn to_document<T: FontResolver>(self, font_resolver: &T) -> TypstDocument {
        let mut builder = TypeDocumentParser::new();
        for font in self.meta.fonts {
            let font_info = TypstFontInfo {
                family: font.family,
                variant: font.variant,
                flags: TypstFontFlags::from_bits(font.flags).unwrap(),
                coverage: font.coverage,
            };

            builder
                .fonts
                .push(font_resolver.get_by_info(&font_info).unwrap());
        }

        let pages = self
            .pages
            .into_iter()
            .map(|f| builder.parse_frame(&f))
            .collect();

        TypstDocument {
            pages,
            title: self.meta.title.map(TypstEcoString::from),
            author: self
                .meta
                .author
                .into_iter()
                .map(TypstEcoString::from)
                .collect(),
        }
    }
}

impl Default for TypeDocumentParser {
    fn default() -> Self {
        Self::new()
    }
}
