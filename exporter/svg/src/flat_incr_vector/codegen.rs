use std::sync::Arc;

use super::vm::{FlatIncrGroupContext, FlatIncrRenderVm};
use crate::{
    ir::AbsoluteRef,
    vector::{SvgText, SvgTextBuilder, SvgTextNode},
};

/// See [`FlatGroupContext`].
impl<'m, C: FlatIncrRenderVm<'m, Resultant = Arc<SvgTextNode>>> FlatIncrGroupContext<C>
    for SvgTextBuilder
{
    fn render_diff_item_ref_at(
        &mut self,
        ctx: &mut C,
        pos: crate::ir::Point,
        item: &AbsoluteRef,
        prev_item: &AbsoluteRef,
    ) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let mut content = vec![];

        if item != prev_item {
            let sub_content = ctx.render_diff_item(item, prev_item);
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

    fn render_diff_reuse_item(&mut self, _ctx: &mut C, item_ref: &AbsoluteRef) {
        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("data-tid", item_ref.as_svg_id("p")),
                ("data-reuse-from", item_ref.as_svg_id("p")),
            ],
            content: vec![],
        })));
    }
}
