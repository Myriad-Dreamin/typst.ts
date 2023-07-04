use std::sync::Arc;

use super::{
    vm::{FlatGroupContext, FlatRenderVm},
    FlatTextItem, GroupRef,
};
use crate::{
    ir::{AbsoluteRef, Scalar},
    vector::{
        codegen::{BuildFillStyleClass, BuildGlyph, DynExportFeature},
        vm::{GroupContext, RenderVm},
        SvgText, SvgTextBuilder, SvgTextNode,
    },
};

/// See [`FlatGroupContext`].
impl<
        'm,
        C: RenderVm<Resultant = Arc<SvgTextNode>>
            + FlatRenderVm<'m, Resultant = Arc<SvgTextNode>>
            + BuildGlyph
            + BuildFillStyleClass
            + DynExportFeature,
    > FlatGroupContext<C> for SvgTextBuilder
{
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: crate::ir::Point, item: &AbsoluteRef) {
        let translate_attr = format!("translate({:.3},{:.3})", pos.x.0, pos.y.0);

        let sub_content = ctx.render_flat_item(item);

        self.content.push(SvgText::Content(Arc::new(SvgTextNode {
            attributes: vec![
                ("transform", translate_attr),
                ("data-tid", item.as_svg_id("p")),
            ],
            content: vec![SvgText::Content(sub_content)],
        })));
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, glyph: &AbsoluteRef) {
        self.render_glyph_inner(ctx, pos, glyph)
    }

    fn render_flat_text_semantics(&mut self, ctx: &mut C, text: &FlatTextItem, width: Scalar) {
        if !ctx.should_render_text_element() {
            return;
        }

        self.render_text_semantics_inner(&text.shape, &text.content.content, width)
    }

    fn with_frame(mut self, _ctx: &mut C, _group: &GroupRef) -> Self {
        self.attributes.push(("class", "typst-group".to_owned()));
        self
    }

    fn with_text(mut self, ctx: &mut C, text: &FlatTextItem) -> Self {
        self.with_text_shape(ctx, &text.shape);
        self
    }

    fn with_reuse(mut self, _ctx: &mut C, v: &AbsoluteRef) -> Self {
        self.attributes.push(("data-reuse-from", v.as_svg_id("g")));
        self
    }
}
