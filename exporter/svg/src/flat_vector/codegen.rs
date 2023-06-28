use std::sync::Arc;

use super::{
    vm::{FlatGroupContext, FlatRenderVm},
    FlatTextItem,
};
use crate::{
    ir::{AbsoluteRef, Scalar},
    vector::{SvgText, SvgTextBuilder, SvgTextNode},
    ExportFeature,
};

/// See [`FlatGroupContext`].
impl<'s, 'm, 't, Feat: ExportFeature> FlatGroupContext for SvgTextBuilder<'s, 'm, 't, Feat> {
    fn render_item_ref_at(&mut self, pos: crate::ir::Point, item: &AbsoluteRef) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = self.t.render_flat_item(item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
            ],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    fn render_glyph_ref(&mut self, pos: Scalar, glyph: &AbsoluteRef) {
        self.render_glyph_ref_inner(pos, glyph)
    }

    fn render_flat_text_semantics(&mut self, text: &FlatTextItem, width: Scalar) {
        if !(Feat::SHOULD_RENDER_TEXT_ELEMENT && self.t.should_render_text_element) {
            return;
        }

        self.render_text_semantics_inner(&text.shape, &text.content.content, width)
    }
}
