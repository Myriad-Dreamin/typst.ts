use std::collections::HashMap;
use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::{ops::Deref, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;
pub use ttf_parser::GlyphId;
use typst::doc::{Document, FrameItem};
use typst::font::Font;
use typst::geom::Axes;
use typst::image::Image as TypstImage;

use crate::artifact::core::FontRef;
use crate::artifact::BuildInfo;
use crate::artifact::{
    font::{FontInfo, FontMetrics},
    image::Image,
};
use crate::artifact_ir::{FontCoverage, TypstFont, TypstFontInfo};
use crate::{make_hash, HashedTrait, StaticHash128};

use super::get_font_coverage_hash;

/// IGlyphProvider extracts the font data from the font.
/// Note (Possibly block unsafe): If a [`Font`] is dummy (lazy loaded),
///   it will block current thread and fetch the font data from the server.
pub trait IGlyphProvider {
    /// With font with glyph id, return the svg document data.
    /// Note: The returned data is possibly compressed.
    /// See [`FontGlyphProvider::svg_glyph`] for the default implementation.
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<[u8]>>;

    /// With font with glyph id, return the bitmap image data.
    /// Optionally, with given ppem, return the best fit bitmap image.
    /// Return the best quality bitmap image if ppem is [`std::u16::MAX`].
    /// See [`FontGlyphProvider::bitmap_glyph`] for the default implementation.
    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(TypstImage, i16, i16)>;

    /// With font with glyph id, return the outline path data.
    /// The returned data is in Path2D format.
    /// See [`FontGlyphProvider::outline_glyph`] for the default implementation.
    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<String>;
}

#[derive(Clone)]
pub struct GlyphProvider(Arc<HashedTrait<dyn IGlyphProvider>>);

impl GlyphProvider {
    pub fn new<T>(provider: T) -> Self
    where
        T: IGlyphProvider + Hash + 'static,
    {
        let hash = make_hash(&provider);
        let provider = Box::new(provider);
        Self(Arc::new(HashedTrait::<dyn IGlyphProvider>::new(
            hash, provider,
        )))
    }
}

impl Deref for GlyphProvider {
    type Target = dyn IGlyphProvider;

    fn deref(&self) -> &Self::Target {
        (*self.0.as_ref()).deref()
    }
}

impl Hash for GlyphProvider {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u128(self.0.get_hash());
    }
}

impl Default for GlyphProvider {
    fn default() -> Self {
        Self::new(FontGlyphProvider::default())
    }
}

/// The default [`IGlyphProvider`] implementation.
/// It uses the local font data to extract the glyph data.
#[derive(Default, Hash)]
pub struct FontGlyphProvider {}

impl IGlyphProvider for FontGlyphProvider {
    /// See [`IGlyphProvider::svg_glyph`] for more information.
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<[u8]>> {
        let font_face = font.ttf();

        Some(font_face.glyph_svg_image(id)?.into())
    }

    /// See [`IGlyphProvider::bitmap_glyph`] for more information.
    /// Note: It converts the data into [`typst::image::Image`] and introduces overhead.
    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(TypstImage, i16, i16)> {
        let font_face = font.ttf();

        let raster = font_face.glyph_raster_image(id, ppem)?;

        // convert to typst's image format
        let glyph_image = TypstImage::new_raw(
            raster.data.into(),
            raster.format.into(),
            Axes::new(raster.width as u32, raster.height as u32),
            None,
        )
        .ok()?;

        Some((glyph_image, raster.x, raster.y))
    }

    /// See [`IGlyphProvider::outline_glyph`] for more information.
    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<String> {
        let font_face = font.ttf();

        // todo: handling no such glyph
        let mut builder = SvgOutlineBuilder(String::new());
        font_face.outline_glyph(id, &mut builder)?;
        Some(builder.0)
    }
}

#[derive(Default)]
struct SvgOutlineBuilder(pub String);

impl ttf_parser::OutlineBuilder for SvgOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "M {} {} ", x, y).unwrap();
    }

    fn line_to(&mut self, x: f32, y: f32) {
        write!(&mut self.0, "L {} {} ", x, y).unwrap();
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    }

    fn close(&mut self) {
        write!(&mut self.0, "Z ").unwrap();
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SvgGlyphInfo {
    #[serde_as(as = "Base64")]
    pub image: Arc<[u8]>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BitmapGlyphInfo {
    pub ppem: u16,
    pub x: i16,
    pub y: i16,
    pub image: Image,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OutlineGlyphInfo {
    pub outline: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GlyphShapeInfo {
    pub id: u16,
    pub svg: Option<Box<SvgGlyphInfo>>,
    pub bitmap: Option<Box<BitmapGlyphInfo>>,
    pub outline: Option<OutlineGlyphInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontGlyphPack {
    pub info: FontInfo,
    pub metrics: FontMetrics,
    pub glyphs: Vec<GlyphShapeInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FontGlyphPackBundle {
    /// metadata about this artifact
    pub build: Option<BuildInfo>,
    /// The page frames.
    pub fonts: Vec<FontGlyphPack>,
}

struct FontGlyphPackBuilder {
    pub info: FontInfo,
    pub metrics: FontMetrics,
    pub glyphs: HashMap<GlyphId, GlyphShapeInfo>,
}

#[derive(Default)]
struct FontGlyphInfoBuilder {
    fonts: Vec<FontGlyphPackBuilder>,
    font_map: HashMap<TypstFontInfo, FontRef>,
    glyph_provider: FontGlyphProvider,
}

impl FontGlyphInfoBuilder {
    pub fn write_font(&mut self, font: &TypstFont) -> FontRef {
        if let Some(font) = self.font_map.get(font.info()) {
            return *font;
        }

        if self.fonts.len() >= u32::MAX as usize {
            panic!("too many fonts");
        }

        let info = font.info();

        let info = FontInfo {
            family: info.family.clone(),
            variant: info.variant,
            flags: info.flags.bits(),
            coverage: FontCoverage::from_vec(info.coverage.iter().take(1).collect()),
            coverage_hash: get_font_coverage_hash(&info.coverage),
        };

        let metrics = (*font.metrics()).into();

        let font_ref = self.fonts.len() as u32;
        self.font_map.insert(font.info().clone(), font_ref);
        self.fonts.push(FontGlyphPackBuilder {
            info,
            metrics,
            glyphs: HashMap::new(),
        });
        font_ref
    }

    fn build(&mut self, doc: &Document) {
        for page in &doc.pages {
            self.write_frame(page);
        }
    }

    fn write_frame(&mut self, frame: &typst::doc::Frame) {
        for (_, item) in frame.items() {
            match item {
                FrameItem::Text(text) => {
                    let font_ref = self.write_font(&text.font);
                    let raw_font = text.font.clone();
                    for glyph in &text.glyphs {
                        let font = &mut self.fonts[font_ref as usize];
                        let id = GlyphId(glyph.id);

                        if font.glyphs.contains_key(&id) {
                            continue;
                        }

                        let item = self.write_glyph(&raw_font, id);
                        let font = &mut self.fonts[font_ref as usize];
                        font.glyphs.insert(id, item);
                    }
                }
                FrameItem::Group(g) => {
                    self.write_frame(&g.frame);
                }
                FrameItem::Shape(..) | FrameItem::Image(..) | FrameItem::Meta(..) => {}
            }
        }
    }

    fn write_glyph(&self, font: &TypstFont, id: GlyphId) -> GlyphShapeInfo {
        let mut glyph_info = GlyphShapeInfo {
            id: id.0,
            ..Default::default()
        };

        let svg = self.glyph_provider.svg_glyph(font, id);
        if let Some(svg) = svg {
            glyph_info.svg = Some(Box::new(SvgGlyphInfo { image: svg }));
        }

        let ppem = std::u16::MAX;

        let bitmap = self.glyph_provider.bitmap_glyph(font, id, ppem);
        if let Some((image, x, y)) = bitmap {
            glyph_info.bitmap = Some(Box::new(BitmapGlyphInfo {
                ppem,
                x,
                y,
                image: image.into(),
            }));
        }

        let outline = self.glyph_provider.outline_glyph(font, id);
        if let Some(outline) = outline {
            glyph_info.outline = Some(OutlineGlyphInfo { outline });
        }

        glyph_info
    }
}

impl From<&Document> for FontGlyphPackBundle {
    fn from(doc: &Document) -> Self {
        let mut builder = FontGlyphInfoBuilder::default();
        builder.build(doc);
        let fonts = builder
            .fonts
            .into_iter()
            .map(|font| FontGlyphPack {
                info: font.info,
                metrics: font.metrics,
                glyphs: font.glyphs.into_values().collect(),
            })
            .collect();
        Self {
            build: Some(BuildInfo {
                compiler: "typst-ts-cli".to_string(),
                version: crate::build_info::VERSION.to_string(),
            }),
            fonts,
        }
    }
}
