use std::sync::Arc;

use crate::{
    ir::{SvgItem, TextItem, TextShape},
    utils::{AbsExt, ToCssExt},
    RenderFeature, SvgRenderTask,
};
use ttf_parser::GlyphId;
use typst::{doc::TextItem as TypstTextItem, geom::Paint};

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Lower a text into item.
    pub(super) fn lower_text(&mut self, text: &TypstTextItem) -> SvgItem {
        let mut glyphs = Vec::with_capacity(text.glyphs.len());
        let mut step = Vec::with_capacity(text.glyphs.len() * 2);
        for glyph in &text.glyphs {
            let id = GlyphId(glyph.id);
            step.push((glyph.x_offset.at(text.size), glyph.x_advance.at(text.size)));
            glyphs.push(crate::ir::GlyphItem::Raw(text.font.clone(), id));
        }

        let glyph_chars: String = text.text
            [text.glyphs[0].range().start..text.glyphs[text.glyphs.len() - 1].range().end]
            .to_string();

        let Paint::Solid(fill) = text.fill;
        let fill = fill.to_css().into();

        let ppem = {
            let pixel_per_unit: f32 = text.size.to_f32();
            let units_per_em = text.font.units_per_em() as f32;
            pixel_per_unit / units_per_em
        };

        SvgItem::Text(Arc::new(TextItem {
            content: glyph_chars.into(),
            step: step.into(),
            glyphs,
            shape: Arc::new(TextShape {
                dir: text.lang.dir(),
                ppem,
                fill,
            }),
        }))
    }
}
