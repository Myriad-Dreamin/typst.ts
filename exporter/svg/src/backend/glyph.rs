use typst_ts_core::{
    font::GlyphProvider,
    vector::{ir, GlyphLowerBuilder},
};

use crate::utils::ToCssExt;

use super::render_image;

pub struct SvgGlyphBuilder {
    pub glyph_provider: GlyphProvider,
}

impl SvgGlyphBuilder {
    pub fn new(glyph_provider: GlyphProvider) -> Self {
        Self { glyph_provider }
    }

    // todo: merge is_image_glyph and render_glyph
    pub fn render_glyph(&mut self, glyph_id: &str, glyph_item: &ir::GlyphItem) -> Option<String> {
        let gp = &self.glyph_provider;
        Self::render_glyph_inner(gp, glyph_id, glyph_item)
    }

    fn render_ligature_attr(ll: u8) -> String {
        // println!("ligature len: {}", ll);
        if ll > 0 {
            format!(r#" data-liga-len="{}""#, ll)
        } else {
            "".to_owned()
        }
    }

    pub fn is_image_glyph(&mut self, glyph_item: &ir::GlyphItem) -> Option<bool> {
        let gp: &GlyphProvider = &self.glyph_provider;
        Self::is_image_glyph_inner(gp, glyph_item)
    }

    #[comemo::memoize]
    fn render_glyph_inner(
        gp: &GlyphProvider,
        glyph_id: &str,
        glyph_item: &ir::GlyphItem,
    ) -> Option<String> {
        if matches!(glyph_item, ir::GlyphItem::Raw(..)) {
            return Self::render_glyph_pure_inner(
                glyph_id,
                &GlyphLowerBuilder::new(gp, true).lower_glyph(glyph_item)?,
            );
        }

        Self::render_glyph_pure_inner(glyph_id, glyph_item)
    }

    #[comemo::memoize]
    fn is_image_glyph_inner(gp: &GlyphProvider, glyph_item: &ir::GlyphItem) -> Option<bool> {
        if matches!(glyph_item, ir::GlyphItem::Raw(..)) {
            return Self::is_image_glyph_pure_inner(
                &GlyphLowerBuilder::new(gp, true).lower_glyph(glyph_item)?,
            );
        }

        Self::is_image_glyph_pure_inner(glyph_item)
    }

    fn render_glyph_pure_inner(glyph_id: &str, glyph_item: &ir::GlyphItem) -> Option<String> {
        match glyph_item {
            ir::GlyphItem::Image(image_glyph) => Self::render_image_glyph(glyph_id, image_glyph),
            ir::GlyphItem::Outline(outline_glyph) => {
                Self::render_outline_glyph(glyph_id, outline_glyph)
            }
            ir::GlyphItem::Raw(..) => unreachable!(),
            ir::GlyphItem::None => None,
        }
    }

    fn is_image_glyph_pure_inner(glyph_item: &ir::GlyphItem) -> Option<bool> {
        match glyph_item {
            ir::GlyphItem::Image(..) => Some(true),
            ir::GlyphItem::Outline(..) => Some(false),
            ir::GlyphItem::Raw(..) => unreachable!(),
            ir::GlyphItem::None => None,
        }
    }

    /// Render an image glyph into the svg text.
    fn render_image_glyph(glyph_id: &str, ig: &ir::ImageGlyphItem) -> Option<String> {
        let transform_style = format!(r#" style="transform: {}""#, ig.ts.to_css());

        let img = render_image(&ig.image.image, ig.image.size, false, &transform_style);

        // Ligature information
        let li = Self::render_ligature_attr(ig.ligature_len);

        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{}" class="image_glyph"{li}>{}</symbol>"#,
            glyph_id, img
        );
        Some(symbol_def)
    }

    /// Render an outline glyph into svg text. This is the "normal" case.
    fn render_outline_glyph(
        glyph_id: &str,
        outline_glyph: &ir::OutlineGlyphItem,
    ) -> Option<String> {
        // Ligature information
        let li = Self::render_ligature_attr(outline_glyph.ligature_len);

        let symbol_def = format!(
            r#"<path id="{}" class="outline_glyph" d="{}"{li}/>"#,
            glyph_id, outline_glyph.d
        );
        Some(symbol_def)
    }
}
