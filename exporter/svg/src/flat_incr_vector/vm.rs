use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};
use std::{ops::Deref, sync::Arc};

use crate::flat_vector::{ir, vm::FlatGroupContext};
use crate::{
    ir::{AbsoluteRef, Point, Scalar},
    vector::{GroupContext, SvgTextBuilder, SvgTextNode, TransformContext},
    ExportFeature, SvgRenderTask,
};

/// A RAII trait for rendering flatten SVG items into underlying context.
pub trait FlatIncrGroupContext: Sized {
    fn render_diff_item_ref_at(&mut self, pos: Point, item: &AbsoluteRef, prev_item: &AbsoluteRef);
    fn render_diff_item_ref(&mut self, item: &AbsoluteRef, prev_item: &AbsoluteRef) {
        self.render_diff_item_ref_at(Point::default(), item, prev_item);
    }

    fn render_diff_reuse_item(&mut self, item_ref: &AbsoluteRef);

    fn with_frame(self, group: &ir::GroupRef) -> Self;
    fn with_text(self, text: &ir::FlatTextItem) -> Self;
    fn with_reuse(self, v: &AbsoluteRef) -> Self;
}

/// A virtual machine for rendering a flatten frame.
/// This is a stateful object that is used to render a frame.
/// The 's lifetime is the lifetime of the virtual machine itself.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait FlatIncrRenderVm<'s, 'm> {
    type Resultant;
    type Group: GroupContext
        + FlatGroupContext
        + FlatIncrGroupContext
        + TransformContext
        + Into<Self::Resultant>;

    fn get_item(&self, value: &AbsoluteRef) -> Option<&'m ir::FlatSvgItem>;

    fn start_flat_group(&'s mut self, value: &AbsoluteRef) -> Self::Group;

    /// Render an item into the a `<g/>` element.
    fn render_diff_item(
        &'s mut self,
        next_abs_ref: &AbsoluteRef,
        prev_abs_ref: &AbsoluteRef,
    ) -> Self::Resultant {
        let next_item: &'m ir::FlatSvgItem = self.get_item(next_abs_ref).unwrap();
        let prev_item = self.get_item(prev_abs_ref);

        let mut group_ctx = self.start_flat_group(next_abs_ref);

        match next_item.deref() {
            ir::FlatSvgItem::Group(group) => {
                let mut group_ctx = group_ctx.with_reuse(prev_abs_ref).with_frame(group);
                Self::render_group_ref(&mut group_ctx, prev_item, group);
                group_ctx
            }
            ir::FlatSvgItem::Item(transformed) => {
                let mut group_ctx = group_ctx.with_reuse(prev_abs_ref).transform(&transformed.0);
                Self::render_diff_transformed_ref(&mut group_ctx, prev_item, transformed);
                group_ctx
            }
            ir::FlatSvgItem::Text(text) => {
                let group_ctx = group_ctx.with_text(text);
                Self::render_flat_text(group_ctx, text)
            }
            ir::FlatSvgItem::Path(path) => {
                group_ctx.render_path(path);
                group_ctx
            }
            ir::FlatSvgItem::Link(link) => {
                group_ctx.render_link(link);
                group_ctx
            }
            ir::FlatSvgItem::Image(image) => {
                group_ctx.render_image(image);
                group_ctx
            }
            ir::FlatSvgItem::None => {
                panic!("FlatRenderVm.RenderFrame.UnknownItem {:?}", next_item)
            }
        }
        .into()
    }

    /// Render a frame group into underlying context.
    fn render_group_ref(
        group_ctx: &mut Self::Group,
        prev_item_: Option<&ir::FlatSvgItem>,
        next: &ir::GroupRef,
    ) {
        if let Some(ir::FlatSvgItem::Group(prev_group)) = prev_item_ {
            let mut unused_prev: BTreeMap<usize, AbsoluteRef> = prev_group
                .0
                .iter()
                .map(|v| v.1.clone())
                .enumerate()
                .collect();
            let reusable: HashSet<AbsoluteRef, RandomState> =
                HashSet::from_iter(prev_group.0.iter().map(|e| e.1.clone()));

            for (_, item_ref) in next.0.iter() {
                if reusable.contains(item_ref) {
                    let remove_key = unused_prev.iter().find(|(_, v)| *v == item_ref);
                    if remove_key.is_none() {
                        continue;
                    }
                    unused_prev.remove(&remove_key.unwrap().0.clone());
                }
            }

            for (pos, item_ref) in next.0.iter() {
                if reusable.contains(item_ref) {
                    group_ctx.render_diff_item_ref_at(*pos, item_ref, item_ref);
                } else if let Some((_, prev_item_re_)) = &unused_prev.pop_first() {
                    group_ctx.render_diff_item_ref_at(*pos, item_ref, prev_item_re_)
                } else {
                    group_ctx.render_item_ref_at(*pos, item_ref)
                }
            }
        } else {
            for (pos, item_ref) in next.0.iter() {
                group_ctx.render_item_ref_at(*pos, item_ref);
            }
        }
    }

    /// Render a transformed frame into underlying context.
    fn render_diff_transformed_ref(
        ts: &mut Self::Group,
        prev_item_: Option<&ir::FlatSvgItem>,
        transformed: &ir::TransformedRef,
    ) {
        let child_ref = &transformed.1;
        if matches!(prev_item_, Some(ir::FlatSvgItem::Item(ir::TransformedRef(_item, prev_ref)))
            if prev_ref == child_ref)
        {
            // assert!(item != &transformed.0);
            ts.render_diff_reuse_item(child_ref);
            return;
        }
        // failed to reuse
        ts.render_item_ref(child_ref);
    }

    /// Render a text into the underlying context.
    fn render_flat_text(group_ctx: Self::Group, text: &ir::FlatTextItem) -> Self::Group {
        let ppem = Scalar(text.shape.ppem.0);

        let mut group_ctx = group_ctx.transform_scale(ppem, -ppem);

        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.0;
            let ts = offset / ppem.0;

            group_ctx.render_glyph_ref(Scalar(ts), glyph);

            x += advance.0;
        }

        group_ctx.render_flat_text_semantics(text, Scalar(x));
        group_ctx
    }
}

impl<'s, 'm: 's, 't: 's, Feat: ExportFeature + 's> FlatIncrRenderVm<'s, 'm>
    for SvgRenderTask<'m, 't, Feat>
{
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder<'s, 'm, 't, Feat>;

    fn get_item(&self, value: &AbsoluteRef) -> Option<&'m ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&'s mut self, v: &AbsoluteRef) -> Self::Group {
        Self::Group {
            t: self,
            attributes: vec![("data-tid", v.as_svg_id("g"))],
            content: Vec::with_capacity(1),
        }
    }
}
