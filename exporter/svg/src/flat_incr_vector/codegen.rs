use std::sync::Arc;

use super::vm::{FlatIncrGroupContext, FlatIncrRenderVm};
use crate::{
    flat_vector::ir,
    ir::AbsoulteRef,
    vector::{SvgText, SvgTextBuilder, SvgTextNode},
    ExportFeature,
};

/// See [`FlatGroupContext`].
impl<'s, 'm, 't, Feat: ExportFeature> FlatIncrGroupContext for SvgTextBuilder<'s, 'm, 't, Feat> {
    fn render_diff_item_ref_at(
        &mut self,
        pos: crate::ir::Point,
        item: &AbsoulteRef,
        prev_item: &AbsoulteRef,
    ) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let mut content = vec![];

        if item != prev_item {
            let sub_content = self.t.render_diff_item(item, prev_item);
            content.push(SvgText::Content(sub_content));
        }

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
                ("data-reuse-from", prev_item.as_svg_id("p")),
            ],
            content,
        })));
    }

    fn render_diff_reuse_item(&mut self, item_ref: &AbsoulteRef) {
        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("data-tid", item_ref.as_svg_id("p")),
                ("data-reuse-from", item_ref.as_svg_id("p")),
            ],
            content: vec![],
        })));
    }

    fn with_frame(mut self, _group: &ir::GroupRef) -> Self {
        self.attributes.push(("class", "typst-group".to_owned()));
        self
    }

    fn with_text(mut self, text: &ir::FlatTextItem) -> Self {
        self.with_text_shape(&text.shape);
        self.attach_debug_info(text.content.span_id);
        self
    }

    fn with_reuse(mut self, v: &AbsoulteRef) -> Self {
        self.attributes.push(("data-reuse-from", v.as_svg_id("g")));
        self
    }
}
