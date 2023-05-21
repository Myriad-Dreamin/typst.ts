use std::collections::HashMap;
use std::slice;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;
use typst::model::Locator;

use crate::artifact::core::BuildInfo;
use crate::typst_affinite_hash;
use crate::FontResolver;

pub mod doc;
use doc::*;

pub mod geom;
use geom::*;

pub mod image;
use image::*;

pub mod core;
use self::core::*;
use self::ligature::LigatureResolver;

pub(crate) mod ligature;

pub use crate::artifact::core::ArtifactMeta;
pub type FontInfo = crate::artifact::font::FontInfo;
pub type TypstFontInfo = crate::artifact::font::TypstFontInfo;
pub type TypstFont = crate::artifact::font::TypstFont;
pub type FontCoverage = crate::artifact::font::FontCoverage;
pub type TypstFontFlags = crate::artifact::font::TypstFontFlags;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArtifactHeader {
    /// Other metadata as json.
    pub metadata: ArtifactMeta,
    /// The page frames.
    pub pages: ItemArray<Frame>,
}

#[derive(Clone, Debug)]
pub struct Artifact {
    /// memory buffer for ItemRef
    pub buffer: Vec<u8>,
    /// Other metadata as json.
    pub metadata: ArtifactMeta,
    /// The page frames.
    pub pages: ItemArray<Frame>,
}

pub struct ArtifactBuilder {
    fonts: Vec<(TypstFontInfo, LigatureResolver)>,
    font_map: HashMap<TypstFontInfo, FontRef>,
    with_ligature: bool,
    buffer: Vec<u8>,
    stat: std::collections::BTreeMap<ItemRefKind, u32>, // for debug
}

impl ArtifactBuilder {
    pub fn new() -> Self {
        Self {
            fonts: vec![],
            font_map: HashMap::default(),
            with_ligature: true,
            buffer: vec![],
            stat: Default::default(),
        }
    }

    pub fn push_item<T: Sized + HasItemRefKind>(&mut self, item: &T) -> ItemRef<T> {
        let idx = self.buffer.len();
        unsafe {
            let raw_item =
                slice::from_raw_parts(item as *const T as *const u8, std::mem::size_of::<T>());
            self.buffer.extend_from_slice(raw_item);
            self.stat
                .entry(T::ITEM_REF_KIND)
                .and_modify(|e| *e += raw_item.len() as u32);
        }
        ItemRef {
            id: idx as u32,
            kind: T::ITEM_REF_KIND,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn push_string(&mut self, s: String) -> ItemRef<String> {
        let idx = self.buffer.len();
        self.buffer.extend_from_slice(s.as_bytes());
        // null terminator
        self.buffer.push(0);
        self.stat
            .entry(ItemRefKind::String)
            .and_modify(|e| *e += s.as_bytes().len() as u32);
        ItemRef {
            id: idx as u32,
            kind: ItemRefKind::String,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn push_array<T: Sized + HasItemRefKind>(&mut self, arr: &Vec<T>) -> ItemArray<T> {
        let start_idx = self.buffer.len();
        for ele in arr {
            self.push_item(ele);
        }
        self.stat
            .entry(T::ITEM_REF_KIND)
            .and_modify(|e| *e += (arr.len() * std::mem::size_of::<T>()) as u32);
        ItemArray {
            start: start_idx as u32,
            size: arr.len() as u32,
            phantom: std::marker::PhantomData,
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
        self.fonts.push((
            TypstFontInfo {
                family: font.info().family.clone(),
                variant: font.info().variant,
                flags: font.info().flags,
                coverage: FontCoverage::from_vec(vec![]),
            },
            LigatureResolver::new(font.ttf()),
        ));
        font_ref
    }

    pub fn write_glyph(&mut self, glyph: TypstGlyph) -> Glyph {
        Glyph {
            id: glyph.id,
            x_advance: glyph.x_advance.into(),
            x_offset: glyph.x_offset.into(),
            span: ((), glyph.span.1), // todo
            range: glyph.range,
        }
    }

    pub fn write_ligature_covered(
        &mut self,
        face: &ttf_parser::Face<'_>,
        font: FontRef,
        text: &TypstTextItem,
    ) {
        let font = &mut self.fonts[font as usize];
        for glyph in &text.glyphs {
            font.1.resolve(face, text, glyph);
        }
    }

    pub fn write_text_item(&mut self, text: &TypstTextItem) -> TextItem {
        let idx = self.write_font(&text.font);
        if self.with_ligature {
            self.write_ligature_covered(text.font.ttf(), idx, text);
        }

        let glyphs: Vec<_> = text
            .clone()
            .glyphs
            .into_iter()
            .map(|g| self.write_glyph(g))
            .collect();

        TextItem {
            font: idx,
            size: text.size.into(),
            fill: text.fill.clone().into(),
            lang: self.push_string(text.lang.as_str().to_string()),
            text: self.push_string(text.text.as_str().to_string()),
            glyphs: self.push_array(&glyphs),
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
        let data = self.push_array(&image.data().to_vec());
        let format = self.push_string(
            match image.format() {
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
        );
        return Image {
            data,
            format,
            width: image.width(),
            height: image.height(),
            alt: image.alt().map(|s| self.push_string(s.to_string())),
        };
    }

    pub fn write_frame_item(&mut self, item: &TypstFrameItem) -> FrameItem {
        match item {
            TypstFrameItem::Text(text) => FrameItem::Text(self.write_text_item(text)),
            TypstFrameItem::Group(group) => FrameItem::Group(self.write_group_item(group)),
            TypstFrameItem::Shape(shape, _) => FrameItem::Shape(shape.clone().into()),
            TypstFrameItem::Image(image, size, _) => {
                FrameItem::Image(self.write_image(image), (*size).into())
            }
            TypstFrameItem::Meta(meta, size) => match meta {
                TypstMeta::Link(dest) => FrameItem::MetaLink(
                    match dest {
                        TypstDestination::Url(url) => {
                            Destination::Url(self.push_string(url.as_str().to_string()))
                        }
                        TypstDestination::Position(pos) => Destination::Position(Position {
                            page: pos.page,
                            point: pos.point.into(),
                        }),
                        TypstDestination::Location(loc) => {
                            // todo: we have no idea to preserve information about the location
                            Destination::Location(
                                self.push_string(format!("{:?}", typst_affinite_hash(loc))),
                            )
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
        let items: Vec<_> = frame
            .items()
            .map(|item| {
                let fi = self.write_frame_item(&item.1);
                ItemWithPos {
                    item: self.push_item(&fi),
                    pos: item.0.into(),
                }
            })
            .collect();
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
            items: self.push_array(&items),
        }
    }

    pub fn build_buffer(&mut self) -> Vec<u8> {
        let mut buffer = Vec::new();
        std::mem::swap(&mut self.buffer, &mut buffer);
        buffer
    }
}

impl From<&TypstDocument> for Artifact {
    fn from(typst_doc: &TypstDocument) -> Self {
        let mut builder = ArtifactBuilder::new();
        builder.stat.insert(ItemRefKind::Frame, 0);
        builder.stat.insert(ItemRefKind::FrameItem, 0);
        builder.stat.insert(ItemRefKind::String, 0);
        builder.stat.insert(ItemRefKind::ItemWithPos, 0);

        let raw_pages = typst_doc
            .pages
            .iter()
            .map(|f| builder.write_frame(f))
            .collect();
        let pages = builder.push_array(&raw_pages);

        println!("stat: {:?}\n", builder.stat);

        let buffer = builder.build_buffer();
        let metadata = ArtifactMeta {
            build: Some(BuildInfo {
                compiler: "typst-ts-cli".to_string(),
                version: crate::build_info::VERSION.to_string(),
            }),
            fonts: builder
                .fonts
                .into_iter()
                .map(|f| {
                    let (info, res) = f;

                    FontInfo {
                        family: info.family,
                        variant: info.variant,
                        flags: info.flags.bits(),
                        coverage: info.coverage,
                        ligatures: res.into_covered(),
                    }
                })
                .collect(),
            title: typst_doc.title.as_ref().map(|s| s.to_string()),
            author: typst_doc.author.iter().map(|s| s.to_string()).collect(),
        };

        Self {
            buffer,
            metadata,
            pages,
        }
    }
}

pub struct TypeDocumentParser<'a> {
    sp: Locator<'static>,
    fonts: Vec<TypstFont>,
    buffer: &'a [u8],
}

impl<'a> TypeDocumentParser<'a> {
    pub fn new(buffer: &'a Vec<u8>) -> Self {
        Self {
            sp: Locator::new(),
            fonts: Vec::new(),
            buffer: buffer.as_ref(),
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
            range: glyph.range.clone(),
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
            lang: TypstLang::from_str(text.lang.as_str(self.buffer)).unwrap(),
            text: text.text.as_str(self.buffer).into(),
            glyphs: text
                .glyphs
                .iter(self.buffer)
                .map(|g| self.parse_glyph(g))
                .collect(),
        }
    }

    pub fn parse_image(&mut self, image: &Image) -> TypstImage {
        TypstImage::new_raw(
            image.data.to_vec(self.buffer).into(),
            match image.format.as_str(self.buffer) {
                "png" => ImageFormat::Raster(RasterFormat::Png),
                "jpg" => ImageFormat::Raster(RasterFormat::Jpg),
                "gif" => ImageFormat::Raster(RasterFormat::Gif),
                "svg" => ImageFormat::Vector(VectorFormat::Svg),
                _ => panic!("Unknown image format {}", image.format.as_str(self.buffer)),
            },
            TypstAxes {
                x: image.width,
                y: image.height,
            },
            image.alt.clone().map(|s| s.as_str(self.buffer).into()),
        )
        .unwrap()
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
                        TypstDestination::Url(TypstEcoString::from(url.as_str(self.buffer)))
                    }
                    Destination::Position(pos) => TypstDestination::Position(TypstPosition {
                        page: pos.page,
                        point: pos.point.into(),
                    }),
                    Destination::Location(loc) => {
                        match self.parse_location(loc.as_str(self.buffer)) {
                            Some(loc) => TypstDestination::Location(loc),
                            None => panic!("Invalid location: {}", loc.as_str(self.buffer)),
                        }
                    }
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
        let items = frame.items.iter(self.buffer);

        for ItemWithPos { pos, item } in items {
            let item = item.deref(self.buffer);
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
        let mut builder = TypeDocumentParser::new(&self.buffer);
        for font in self.metadata.fonts {
            let font_info = TypstFontInfo {
                family: font.family,
                variant: font.variant,
                flags: TypstFontFlags::from_bits(font.flags).unwrap(),
                coverage: font.coverage,
            };

            // todo: font alternative
            let idx = font_resolver
                .font_book()
                .select_fallback(Some(&font_info), font.variant, "0")
                .unwrap();
            builder.fonts.push(font_resolver.font(idx).unwrap());
        }

        let pages = self
            .pages
            .iter(&self.buffer)
            .map(|f| builder.parse_frame(f))
            .collect();

        TypstDocument {
            pages,
            title: self.metadata.title.map(TypstEcoString::from),
            author: self
                .metadata
                .author
                .into_iter()
                .map(TypstEcoString::from)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::artifact_ir::*;

    fn build_simple_refs(builder: &mut ArtifactBuilder) -> ItemArray<FrameItem> {
        let lang_str = builder.push_string("en".into());
        let text_src = builder.push_string("W".to_string());
        let glyphs = builder.push_array(&vec![Glyph {
            id: 45,
            x_advance: TypstEm::one().into(),
            x_offset: TypstEm::one().into(),
            span: ((), 0),
            range: 0..1,
        }]);

        let item1 = builder.push_item(&FrameItem::Text(TextItem {
            font: 77,
            size: TypstAbs::zero().into(),
            fill: Paint::Solid(Color::Rgba(RgbaColor {
                r: 3,
                g: 4,
                b: 5,
                a: 6,
            })),
            text: text_src,
            lang: lang_str,
            glyphs,
        }));

        let item2 = builder.push_item(&FrameItem::Shape(Shape {
            fill: None,
            stroke: None,
            geometry: Geometry::Rect(Size {
                x: TypstAbs::zero().into(),
                y: TypstAbs::zero().into(),
            }),
        }));

        let items = vec![item1, item2];

        ItemArray {
            start: items.first().map(|x| x.id).unwrap(),
            size: items.len() as u32,
            phantom: std::marker::PhantomData,
        }
    }

    #[test]
    fn test_item_ref_array() {
        let mut builder = ArtifactBuilder::new();
        let refs = build_simple_refs(&mut builder);
        assert_eq!(refs.len(), 2);

        let mut it = refs.iter(&builder.buffer);
        assert_eq!(it.len(), 2);
        if let Some(FrameItem::Text(x)) = it.next() {
            assert_eq!(x.glyphs.len(), 1);
            if let Some(x) = x.glyphs.iter(&builder.buffer).next() {
                assert_eq!(x.range, 0..1);
            } else {
                panic!("Expected glyph item");
            }
        } else {
            panic!("Expected text item");
        }

        if let Some(FrameItem::Shape(x)) = it.next() {
            assert_eq!(
                x.geometry,
                Geometry::Rect(Size {
                    x: TypstAbs::zero().into(),
                    y: TypstAbs::zero().into(),
                })
            );
        } else {
            panic!("Expected shape item");
        }

        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_item_ref_option() {
        let mut builder = ArtifactBuilder::new();

        let raw_item = builder.push_item(&Frame {
            size: Axes {
                x: TypstAbs::zero().into(),
                y: TypstAbs::zero().into(),
            },
            baseline: Some(TypstAbs::raw(1.2).into()),
            items: Default::default(),
        });

        let item = raw_item.deref(&builder.buffer);
        assert!(matches!(item.baseline, Some(_)));
    }
}

impl Default for ArtifactBuilder {
    fn default() -> Self {
        Self::new()
    }
}
