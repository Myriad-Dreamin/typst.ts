use reflexo_typst2vec::{ir, utils::ToCssExt};

use super::render_image;

#[derive(Default)]
pub struct SvgGlyphBuilder {}

impl SvgGlyphBuilder {
    pub fn new() -> Self {
        Self {}
    }

    // todo: merge is_image_glyph and render_glyph
    pub fn render_glyph(
        &mut self,
        glyph_id: &str,
        glyph_item: &ir::FlatGlyphItem,
    ) -> Option<String> {
        Self::render_glyph_inner(glyph_id, glyph_item)
    }

    fn render_ligature_attr(ll: u8) -> String {
        // eprintln!("ligature len: {}", ll);
        if ll > 0 {
            format!(r#" data-liga-len="{ll}""#)
        } else {
            "".to_owned()
        }
    }

    #[comemo::memoize]
    fn render_glyph_inner(glyph_id: &str, glyph_item: &ir::FlatGlyphItem) -> Option<String> {
        match glyph_item {
            ir::FlatGlyphItem::Image(image_glyph) => {
                Self::render_image_glyph(glyph_id, image_glyph)
            }
            ir::FlatGlyphItem::Outline(outline_glyph) => {
                Self::render_outline_glyph(glyph_id, outline_glyph)
            }
            ir::FlatGlyphItem::None => None,
        }
    }

    /// Render an image glyph into the svg text.
    fn render_image_glyph(glyph_id: &str, ig: &ir::ImageGlyphItem) -> Option<String> {
        let transform_style = format!(r#" style="transform: {}""#, ig.ts.to_css());

        let img = render_image(&ig.image.image, ig.image.size, false, &transform_style);

        // Ligature information
        let li = Self::render_ligature_attr(ig.ligature_len);

        let symbol_def = format!(
            r#"<symbol overflow="visible" id="{glyph_id}" class="image_glyph"{li}>{img}</symbol>"#
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
