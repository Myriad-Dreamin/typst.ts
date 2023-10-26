use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};

use crate::hash::Fingerprint;

use super::flat_ir as ir;
use super::ir::{FontIndice, GlyphRef, Size, Transform};
use super::vm::RenderState;
use super::{
    ir::{Point, Scalar},
    vm::{GroupContext, TransformContext},
};

/// A RAII trait for rendering flatten SVG items into underlying context.
pub trait FlatGroupContext<C>: Sized {
    fn render_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        pos: Point,
        item: &Fingerprint,
    );
    fn render_item_ref(&mut self, state: RenderState, ctx: &mut C, item: &Fingerprint) {
        self.render_item_ref_at(state, ctx, Point::default(), item);
    }

    fn render_glyph_ref(&mut self, _ctx: &mut C, _pos: Scalar, _item: &GlyphRef) {}

    fn render_flat_text_semantics(
        &mut self,
        _ctx: &mut C,
        _text: &ir::FlatTextItem,
        _width: Scalar,
    ) {
    }

    fn with_frame(self, _ctx: &mut C, _group: &ir::GroupRef) -> Self {
        self
    }
    fn with_text(
        self,
        _ctx: &mut C,
        _text: &ir::FlatTextItem,
        _fill_key: &Fingerprint,
        _state: RenderState,
    ) -> Self {
        self
    }
    fn with_reuse(self, _ctx: &mut C, _v: &Fingerprint) -> Self {
        self
    }
}

/// A virtual machine for rendering a flatten frame.
/// This is a stateful object that is used to render a frame.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait FlatRenderVm<'m>: Sized + FontIndice<'m> {
    type Resultant;
    type Group: GroupContext<Self>
        + FlatGroupContext<Self>
        + TransformContext<Self>
        + Into<Self::Resultant>;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::FlatSvgItem>;

    fn start_flat_group(&mut self, value: &Fingerprint) -> Self::Group;

    fn start_flat_frame(&mut self, value: &Fingerprint, _group: &ir::GroupRef) -> Self::Group {
        self.start_flat_group(value)
    }

    fn start_flat_text(
        &mut self,
        _state: RenderState,
        value: &Fingerprint,
        _text: &ir::FlatTextItem,
    ) -> Self::Group {
        self.start_flat_group(value)
    }

    #[doc(hidden)]
    /// Default implemenetion to render an item into the a `<g/>` element.
    fn _render_flat_item(&mut self, state: RenderState, abs_ref: &Fingerprint) -> Self::Resultant {
        let item: &'m ir::FlatSvgItem = self.get_item(abs_ref).unwrap();
        match &item {
            ir::FlatSvgItem::Group(group, sz) => self.render_group_ref(state, abs_ref, group, sz),
            ir::FlatSvgItem::Item(transformed) => {
                self.render_transformed_ref(state, abs_ref, transformed)
            }
            ir::FlatSvgItem::Text(text) => {
                let mut g = self.start_flat_text(state, abs_ref, text);
                g = self.render_flat_text(state, g, abs_ref, text);

                g.into()
            }
            ir::FlatSvgItem::Path(path) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_path(state, self, path, abs_ref);
                g.into()
            }
            ir::FlatSvgItem::Link(link) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_link(self, link);
                g.into()
            }
            ir::FlatSvgItem::Image(image) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_image(self, image);
                g.into()
            }
            ir::FlatSvgItem::Gradient(..) | ir::FlatSvgItem::None => {
                panic!("FlatRenderVm.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render an item into the a `<g/>` element.
    fn render_flat_item(&mut self, state: RenderState, abs_ref: &Fingerprint) -> Self::Resultant {
        self._render_flat_item(state, abs_ref)
    }

    /// Render a frame group into underlying context.
    fn render_group_ref(
        &mut self,
        mut state: RenderState,
        abs_ref: &Fingerprint,
        group: &ir::GroupRef,
        sz: &Option<Size>,
    ) -> Self::Resultant {
        let mut group_ctx = self.start_flat_frame(abs_ref, group);

        if let Some(sz) = sz {
            state = state.with_transform(Transform::identity()).with_size(*sz);
        }

        for (pos, item_ref) in group.0.iter() {
            // let item = self.get_item(&item_ref).unwrap();
            group_ctx.render_item_ref_at(state.pre_translate(*pos), self, *pos, item_ref);
        }

        group_ctx.into()
    }

    /// Render a transformed frame into underlying context.
    fn render_transformed_ref(
        &mut self,
        state: RenderState,
        abs_ref: &Fingerprint,
        transformed: &ir::TransformedRef,
    ) -> Self::Resultant {
        let mut ts = self
            .start_flat_group(abs_ref)
            .transform(self, &transformed.0);

        let item_ref = &transformed.1;
        // let item = self.get_item(&item_ref).unwrap();
        ts.render_item_ref(state.pre_apply(&transformed.0), self, item_ref);
        ts.into()
    }

    /// Render a text into the underlying context.
    fn render_flat_text(
        &mut self,
        _state: RenderState,
        mut group_ctx: Self::Group,
        _abs_ref: &Fingerprint,
        text: &ir::FlatTextItem,
    ) -> Self::Group {
        // upem is the unit per em defined in the font.
        let font = self.get_font(&text.font).unwrap();
        let upem = Scalar(font.unit_per_em.0);

        // Rescale the font size and put glyphs into the group.
        group_ctx = text.shape.add_transform(self, group_ctx, upem);
        text.render_glyphs(upem, |x, g| {
            group_ctx.render_glyph_ref(self, x, g);
        });
        group_ctx
    }
}

/// A RAII trait for rendering flatten SVG items into underlying context.
pub trait FlatIncrGroupContext<C>: Sized {
    fn render_diff_item_ref_at(
        &mut self,
        state: RenderState,
        ctx: &mut C,

        pos: Point,
        item: &Fingerprint,
        prev_item: &Fingerprint,
    );
    fn render_diff_item_ref(
        &mut self,
        state: RenderState,
        ctx: &mut C,
        item: &Fingerprint,
        prev_item: &Fingerprint,
    ) {
        self.render_diff_item_ref_at(state, ctx, Point::default(), item, prev_item);
    }
}

/// A virtual machine that diffs and renders a flatten frame.
/// This is a stateful object that is used to render a frame.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait FlatIncrRenderVm<'m>: FlatRenderVm<'m> + Sized
where
    Self::Group: FlatIncrGroupContext<Self>,
{
    #[doc(hidden)]
    /// Default implemenetion to Render an item into the a `<g/>` element.
    fn _render_diff_item(
        &mut self,
        state: RenderState,
        next_abs_ref: &Fingerprint,
        prev_abs_ref: &Fingerprint,
    ) -> Self::Resultant {
        let next_item: &'m ir::FlatSvgItem = self.get_item(next_abs_ref).unwrap();
        let prev_item = self.get_item(prev_abs_ref);

        let mut group_ctx = self.start_flat_group(next_abs_ref);

        match &next_item {
            ir::FlatSvgItem::Group(group, sz) => {
                let mut group_ctx = group_ctx
                    .with_reuse(self, prev_abs_ref)
                    .with_frame(self, group);
                self.render_diff_group_ref(state, &mut group_ctx, prev_item, group, sz);
                group_ctx
            }
            ir::FlatSvgItem::Item(transformed) => {
                let mut group_ctx = group_ctx
                    .with_reuse(self, prev_abs_ref)
                    .transform(self, &transformed.0);
                self.render_diff_transformed_ref(state, &mut group_ctx, prev_item, transformed);
                group_ctx
            }
            ir::FlatSvgItem::Text(text) => {
                let group_ctx = group_ctx.with_text(self, text, next_abs_ref, state);
                self.render_diff_flat_text(state, group_ctx, next_abs_ref, prev_abs_ref, text)
            }
            ir::FlatSvgItem::Path(path) => {
                group_ctx.render_path(state, self, path, next_abs_ref);
                group_ctx
            }
            ir::FlatSvgItem::Link(link) => {
                group_ctx.render_link(self, link);
                group_ctx
            }
            ir::FlatSvgItem::Image(image) => {
                group_ctx.render_image(self, image);
                group_ctx
            }
            ir::FlatSvgItem::Gradient(..) | ir::FlatSvgItem::None => {
                panic!("FlatRenderVm.RenderFrame.UnknownItem {:?}", next_item)
            }
        }
        .into()
    }

    /// Render an item into the a `<g/>` element.
    fn render_diff_item(
        &mut self,
        state: RenderState,
        next_abs_ref: &Fingerprint,
        prev_abs_ref: &Fingerprint,
    ) -> Self::Resultant {
        self._render_diff_item(state, next_abs_ref, prev_abs_ref)
    }

    /// Render a frame group into underlying context.
    fn render_diff_group_ref(
        &mut self,
        mut state: RenderState,
        group_ctx: &mut Self::Group,
        prev_item_: Option<&ir::FlatSvgItem>,
        next: &ir::GroupRef,
        sz: &Option<Size>,
    ) {
        if let Some(sz) = sz {
            state = state.with_size(*sz);
        }

        if let Some(ir::FlatSvgItem::Group(prev_group, _)) = prev_item_ {
            let mut unused_prev: BTreeMap<usize, Fingerprint> =
                prev_group.0.iter().map(|v| v.1).enumerate().collect();
            let reusable: HashSet<Fingerprint, RandomState> =
                HashSet::from_iter(prev_group.0.iter().map(|e| e.1));

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
                let state = state.pre_translate(*pos);
                if reusable.contains(item_ref) {
                    group_ctx.render_diff_item_ref_at(state, self, *pos, item_ref, item_ref);
                } else if let Some((_, prev_item_re_)) = &unused_prev.pop_first() {
                    group_ctx.render_diff_item_ref_at(state, self, *pos, item_ref, prev_item_re_)
                } else {
                    group_ctx.render_item_ref_at(state, self, *pos, item_ref)
                }
            }
        } else {
            for (pos, item_ref) in next.0.iter() {
                group_ctx.render_item_ref_at(state.pre_translate(*pos), self, *pos, item_ref);
            }
        }
    }

    /// Render a transformed frame into underlying context.
    fn render_diff_transformed_ref(
        &mut self,
        state: RenderState,
        ts: &mut Self::Group,
        prev_item_: Option<&ir::FlatSvgItem>,
        transformed: &ir::TransformedRef,
    ) {
        let child_ref = &transformed.1;
        let state = state.pre_apply(&transformed.0);
        if matches!(prev_item_, Some(ir::FlatSvgItem::Item(ir::TransformedRef(_item, prev_ref)))
            if prev_ref == child_ref)
        {
            // assert!(item != &transformed.0);
            ts.render_diff_item_ref_at(state, self, Point::default(), child_ref, child_ref);
            return;
        }
        // failed to reuse
        ts.render_item_ref(state, self, child_ref);
    }

    /// Render a diff text into the underlying context.
    fn render_diff_flat_text(
        &mut self,
        state: RenderState,
        group_ctx: Self::Group,
        next_abs_ref: &Fingerprint,
        _prev_abs_ref: &Fingerprint,
        text: &ir::FlatTextItem,
    ) -> Self::Group {
        self.render_flat_text(state, group_ctx, next_abs_ref, text)
    }
}
