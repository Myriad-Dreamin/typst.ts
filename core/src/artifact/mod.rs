use siphasher::sip128::Hasher128;
use std::collections::HashMap;
use std::str::FromStr;

pub mod doc;
use doc::*;

pub mod font;
use font::*;

pub mod geom;
use geom::*;

pub mod image;
use image::*;

pub mod core;
use self::core::*;

use serde::Deserialize;
use serde::Serialize;
use typst::model::StabilityProvider;

use crate::FontResolver;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Artifact {
    /// The page frames.
    pub pages: Vec<Frame>,
    /// The document used fonts.
    pub fonts: Vec<FontInfo>,
    /// The document's title.
    pub title: Option<EcoString>,
    /// The document's author.
    pub author: Vec<EcoString>,
}

pub struct ArtifactBuilder {
    fonts: Vec<FontInfo>,
    font_map: HashMap<FontInfo, FontRef>,
}

fn calculate_hash<T: std::hash::Hash>(t: &T) -> u128 {
    let mut s = siphasher::sip128::SipHasher::new();
    t.hash(&mut s);
    s.finish128().as_u128()
}

impl ArtifactBuilder {
    pub fn new() -> Self {
        Self {
            fonts: vec![],
            font_map: HashMap::default(),
        }
    }

    pub fn write_font(&mut self, font: &TypstFont) -> FontRef {
        if let Some(&font) = self.font_map.get(font.info()) {
            return font;
        }

        if self.fonts.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let font_ref = self.fonts.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.fonts.push(FontInfo {
            family: font.info().family.clone(),
            variant: font.info().variant.clone(),
            flags: font.info().flags,
            coverage: FontCoverage::from_vec(vec![]),
        });
        font_ref
    }

    pub fn write_span(&mut self, _span: TypstSpan) -> SpanRef {
        // todo
        ()
    }

    pub fn write_glyph(&mut self, glyph: TypstGlyph) -> Glyph {
        Glyph {
            id: glyph.id,
            x_advance: glyph.x_advance.into(),
            x_offset: glyph.x_offset.into(),
            c: glyph.c,
            span: self.write_span(glyph.span),
            offset: glyph.offset,
        }
    }

    pub fn write_text_item(&mut self, text: &TypstTextItem) -> TextItem {
        TextItem {
            font: self.write_font(&text.font),
            size: text.size.into(),
            fill: text.fill.clone().into(),
            lang: text.lang.as_str().to_string(),
            glyphs: text
                .clone()
                .glyphs
                .into_iter()
                .map(|g| self.write_glyph(g))
                .collect(),
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
        return Image {
            data: image.data().to_vec(),
            format: match image.format() {
                ImageFormat::Raster(r) => match r {
                    RasterFormat::Png => "png",
                    RasterFormat::Jpg => "jpg",
                    RasterFormat::Gif => "gif",
                },
                ImageFormat::Vector(v) => match v {
                    VectorFormat::Svg => "svg",
                },
            }
            .to_string(),
            width: image.width(),
            height: image.height(),
        };
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
                            // todo: we have no idea to preserve information about the location
                            Destination::Location(format!("{:?}", calculate_hash(loc)))
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

impl From<TypstDocument> for Artifact {
    fn from(typst_doc: TypstDocument) -> Self {
        let mut builder = ArtifactBuilder::new();

        let pages = typst_doc
            .pages
            .into_iter()
            .map(|f| builder.write_frame(&f))
            .collect();

        Self {
            pages,
            fonts: builder.fonts,
            title: typst_doc.title.map(|s| s.to_string()),
            author: typst_doc
                .author
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

pub struct TypeDocumentParser {
    sp: StabilityProvider,
    fonts: Vec<TypstFont>,
}

impl TypeDocumentParser {
    pub fn new() -> Self {
        Self {
            sp: StabilityProvider::new(),
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
            c: glyph.c,
            span: TypstSpan::detached(),
            offset: glyph.offset,
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
            glyphs: text.glyphs.iter().map(|g| self.parse_glyph(g)).collect(),
        }
    }

    pub fn parse_image(&mut self, image: &Image) -> TypstImage {
        TypstImage::new(
            image.data.clone().into(),
            match image.format.as_str() {
                "png" => ImageFormat::Raster(RasterFormat::Png),
                "jpg" => ImageFormat::Raster(RasterFormat::Jpg),
                "gif" => ImageFormat::Raster(RasterFormat::Gif),
                "svg" => ImageFormat::Vector(VectorFormat::Svg),
                _ => panic!("Unknown image format {}", image.format),
            },
        )
        .unwrap()
    }

    pub fn parse_location(&mut self, loc: &String) -> Option<TypstLocation> {
        let loc_hash = u128::from_str_radix(loc, 10).ok()?;
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
                        TypstDestination::Url(TypstEcoString::from(url.clone()))
                    }
                    Destination::Position(pos) => TypstDestination::Position(TypstPosition {
                        page: pos.page,
                        point: pos.point.into(),
                    }),
                    Destination::Location(loc) => match self.parse_location(loc) {
                        Some(loc) => TypstDestination::Location(loc),
                        None => panic!("Invalid location: {}", loc),
                    },
                };

                TypstFrameItem::Meta(TypstMeta::Link(dest), (*size).into())
            }
            FrameItem::None => panic!("None frame item"),
        }
    }

    pub fn parse_frame(&mut self, frame: &Frame) -> TypstFrame {
        let mut parsed_frame = TypstFrame::new(frame.size.into());

        for (pos, item) in frame.items.iter() {
            match item {
                FrameItem::None => continue,
                _ => {
                    parsed_frame.push(pos.clone().into(), self.parse_frame_item(item));
                }
            };
        }

        parsed_frame
    }
}

impl Artifact {
    pub fn to_document(self, font_mgr: &impl FontResolver) -> TypstDocument {
        let mut builder = TypeDocumentParser::new();
        for font in self.fonts {
            // todo: font alternative
            let idx = font_mgr
                .font_book()
                .select_fallback(Some(&font), font.variant, "0")
                .unwrap();
            builder.fonts.push(font_mgr.font(idx).unwrap());
        }

        let pages = self
            .pages
            .into_iter()
            .map(|f| builder.parse_frame(&f))
            .collect();

        TypstDocument {
            pages,
            title: self.title.map(|s| TypstEcoString::from(s)),
            author: self
                .author
                .into_iter()
                .map(|s| TypstEcoString::from(s))
                .collect(),
        }
    }
}
