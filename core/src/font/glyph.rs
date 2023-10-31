use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::{ops::Deref, sync::Arc};

pub use ttf_parser::GlyphId;
use typst::font::Font;
// use typst::geom::Axes;
use typst::image::{Image as TypstImage, RasterFormat};

use crate::hash::item_hash128;
use crate::{HashedTrait, StaticHash128};

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
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new<T>(provider: T) -> Self
    where
        T: IGlyphProvider + Hash + 'static,
    {
        let hash = item_hash128(&provider);
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
    /// Note: It converts the data into [`typst::image::Image`] and introduces
    /// overhead.
    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(TypstImage, i16, i16)> {
        let font_face = font.ttf();

        let raster = font_face.glyph_raster_image(id, ppem)?;

        // todo: more raster image support?
        if raster.format != ttf_parser::RasterImageFormat::PNG {
            return None;
        }

        // convert to typst's image format
        // todo: verify result
        let glyph_image = TypstImage::new(
            raster.data.into(),
            RasterFormat::Png.into(),
            // Axes::new(raster.width as u32, raster.height as u32),
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
