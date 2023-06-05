use std::fmt::Write;
use std::hash::{Hash, Hasher};
use std::{ops::Deref, sync::Arc};

use ttf_parser::GlyphId;
use typst::font::Font;
use typst::geom::Axes;
use typst::image::Image;

use crate::{make_hash, HashedTrait, StaticHash128};

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

pub trait IGlyphProvider {
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<[u8]>>;
    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(Image, i16, i16)>;
    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<String>;
}

#[derive(Default, Hash)]
pub struct FontGlyphProvider {}

impl IGlyphProvider for FontGlyphProvider {
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<[u8]>> {
        let data = font.ttf().glyph_svg_image(id)?;
        Some(data.into())
    }

    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(Image, i16, i16)> {
        let raster = font.ttf().glyph_raster_image(id, ppem)?;

        Some((
            Image::new_raw(
                raster.data.into(),
                raster.format.into(),
                Axes::new(raster.width as u32, raster.height as u32),
                None,
            )
            .ok()?,
            raster.x,
            raster.y,
        ))
    }

    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<String> {
        // todo: handling no such glyph
        let mut builder = SvgOutlineBuilder(String::new());
        font.ttf().outline_glyph(id, &mut builder)?;
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
