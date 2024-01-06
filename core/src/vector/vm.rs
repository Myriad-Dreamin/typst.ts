use crate::hash::{item_hash128, Fingerprint};
use std::collections::hash_map::RandomState;
use std::collections::{BTreeMap, HashSet};

use super::ir::{self, Abs, Axes, FontIndice, FontItem, Point, Ratio, Scalar, Size, Transform};

/// A build pattern for applying transforms to the group of items.
/// See [`ir::Transform`].
pub trait TransformContext<C>: Sized {
    fn transform_matrix(self, ctx: &mut C, matrix: &ir::Transform) -> Self;
    fn transform_translate(self, ctx: &mut C, matrix: Axes<Abs>) -> Self;
    fn transform_scale(self, ctx: &mut C, x: Ratio, y: Ratio) -> Self;
    fn transform_rotate(self, ctx: &mut C, matrix: Scalar) -> Self;
    fn transform_skew(self, ctx: &mut C, matrix: (Ratio, Ratio)) -> Self;
    fn transform_clip(self, ctx: &mut C, matrix: &ir::PathItem) -> Self;

    /// See [`ir::TransformItem`].
    fn transform(self, ctx: &mut C, transform: &ir::TransformItem) -> Self {
        match transform {
            ir::TransformItem::Matrix(transform) => self.transform_matrix(ctx, transform.as_ref()),
            ir::TransformItem::Translate(transform) => {
                self.transform_translate(ctx, *transform.clone())
            }
            ir::TransformItem::Scale(transform) => {
                self.transform_scale(ctx, transform.0, transform.1)
            }
            ir::TransformItem::Rotate(transform) => self.transform_rotate(ctx, *transform.clone()),
            ir::TransformItem::Skew(transform) => self.transform_skew(ctx, *transform.clone()),
            ir::TransformItem::Clip(transform) => self.transform_clip(ctx, transform.as_ref()),
        }
    }
}

/// A RAII trait for rendering vector items into underlying context.
pub trait GroupContext<C>: Sized {
    /// attach shape of the text to the node using css rules.
    fn with_text_shape(
        &mut self,
        _ctx: &mut C,
        _upem: Scalar,
        _shape: &ir::TextShape,
        _fill_key: &Fingerprint,
        _state: RenderState,
    ) {
    }

    /// Render a geometrical shape into underlying context.
    fn render_path(
        &mut self,
        _state: RenderState,
        _ctx: &mut C,
        _path: &ir::PathItem,
        _abs_ref: &Fingerprint,
    ) {
    }

    /// Render a semantic link into underlying context.
    fn render_link(&mut self, _ctx: &mut C, _link: &ir::LinkItem) {}

    /// Render an image into underlying context.
    fn render_image(&mut self, _ctx: &mut C, _image_item: &ir::ImageItem) {}

    fn attach_debug_info(&mut self, _ctx: &mut C, _span_id: u64) {}

    /// Render a semantic link into underlying context.
    fn render_content_hint(&mut self, _ctx: &mut C, _ch: char) {}

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

    fn render_glyph_ref(&mut self, _ctx: &mut C, _pos: Scalar, _font: &FontItem, _glyph_id: u32) {}

    fn render_flat_text_semantics(&mut self, _ctx: &mut C, _text: &ir::TextItem, _width: Scalar) {}

    fn with_frame(self, _ctx: &mut C, _group: &ir::GroupRef) -> Self {
        self
    }
    fn with_text(
        self,
        _ctx: &mut C,
        _text: &ir::TextItem,
        _fill_key: &Fingerprint,
        _state: RenderState,
    ) -> Self {
        self
    }
    fn with_reuse(self, _ctx: &mut C, _v: &Fingerprint) -> Self {
        self
    }
}

/// A RAII trait for rendering flatten SVG items into underlying context.
pub trait IncrGroupContext<C>: Sized {
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

/// Contextual information for rendering.
#[derive(Clone, Copy, Hash)]
pub struct RenderState {
    /// The transform of the current item.
    pub transform: Transform,
    /// The size of the first hard frame in the hierarchy.
    pub size: Size,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            transform: Transform::identity(),
            size: Size::default(),
        }
    }
}

impl RenderState {
    pub fn new_size(size: Size) -> Self {
        Self {
            transform: Transform::identity(),
            size,
        }
    }

    /// Pre translate the current item's transform.
    pub fn pre_translate(self, pos: Point) -> Self {
        self.pre_concat(Transform::from_translate(pos.x, pos.y))
    }

    /// Pre concat the current item's transform.
    pub fn pre_concat(self, transform: Transform) -> Self {
        Self {
            transform: self.transform.pre_concat(transform),
            ..self
        }
    }

    /// Sets the size of the first hard frame in the hierarchy.
    pub fn with_size(self, size: Size) -> Self {
        Self { size, ..self }
    }

    /// Sets the current item's transform.
    pub fn with_transform(self, transform: Transform) -> Self {
        Self { transform, ..self }
    }

    pub fn pre_apply(self, transform: &ir::TransformItem) -> RenderState {
        match transform {
            ir::TransformItem::Matrix(transform) => self.pre_concat(**transform),
            ir::TransformItem::Translate(transform) => {
                self.pre_concat(Transform::from_translate(transform.x, transform.y))
            }
            ir::TransformItem::Scale(transform) => {
                self.pre_concat(Transform::from_scale(transform.0, transform.1))
            }
            ir::TransformItem::Rotate(_transform) => {
                todo!()
            }
            ir::TransformItem::Skew(transform) => {
                self.pre_concat(Transform::from_skew(transform.0, transform.1))
            }
            ir::TransformItem::Clip(_transform) => self,
        }
    }

    pub fn inv_transform(&self) -> Transform {
        self.transform.invert().unwrap()
    }

    pub fn body_inv_transform(&self) -> Transform {
        Transform::from_scale(self.size.x, self.size.y)
            .post_concat(self.transform.invert().unwrap())
    }

    pub fn at(&self, pos: &Fingerprint) -> Fingerprint {
        // todo: performance
        let item = (*self, *pos);
        Fingerprint::from_u128(item_hash128(&item))
    }
}

/// A virtual machine for rendering a frame.
/// This is a stateful object that is used to render a frame.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait RenderVm<'m>: Sized + FontIndice<'m> {
    type Resultant;
    type Group: GroupContext<Self> + TransformContext<Self> + Into<Self::Resultant>;

    fn get_item(&self, value: &Fingerprint) -> Option<&'m ir::VecItem>;

    fn start_flat_group(&mut self, value: &Fingerprint) -> Self::Group;

    fn start_flat_frame(&mut self, value: &Fingerprint, _group: &ir::GroupRef) -> Self::Group {
        self.start_flat_group(value)
    }

    fn start_flat_text(
        &mut self,
        _state: RenderState,
        value: &Fingerprint,
        _text: &ir::TextItem,
    ) -> Self::Group {
        self.start_flat_group(value)
    }

    #[doc(hidden)]
    /// Default implemenetion to render an item into the a `<g/>` element.
    fn _render_flat_item(&mut self, state: RenderState, abs_ref: &Fingerprint) -> Self::Resultant {
        let item: &'m ir::VecItem = self.get_item(abs_ref).unwrap();
        match &item {
            ir::VecItem::Group(group, sz) => self.render_group_ref(state, abs_ref, group, sz),
            ir::VecItem::Item(transformed) => {
                self.render_transformed_ref(state, abs_ref, transformed)
            }
            ir::VecItem::Text(text) => {
                let mut g = self.start_flat_text(state, abs_ref, text);
                g = self.render_flat_text(state, g, abs_ref, text);

                g.into()
            }
            ir::VecItem::Path(path) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_path(state, self, path, abs_ref);
                g.into()
            }
            ir::VecItem::Link(link) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_link(self, link);
                g.into()
            }
            ir::VecItem::Image(image) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_image(self, image);
                g.into()
            }
            ir::VecItem::ContentHint(c) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_content_hint(self, *c);
                g.into()
            }
            ir::VecItem::Gradient(..) | ir::VecItem::Pattern(..) | ir::VecItem::None => {
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
        text: &ir::TextItem,
    ) -> Self::Group {
        // upem is the unit per em defined in the font.
        let font = self.get_font(&text.shape.font).unwrap();
        let upem = Scalar(font.unit_per_em.0);

        // Rescale the font size and put glyphs into the group.
        group_ctx = text.shape.add_transform(self, group_ctx, upem);
        let mut _width = 0f32;
        for (x, g) in text.render_glyphs(upem, &mut _width) {
            group_ctx.render_glyph_ref(self, x, font, g);
        }

        group_ctx
    }
}

/// A virtual machine that diffs and renders a frame.
/// This is a stateful object that is used to render a frame.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait IncrRenderVm<'m>: RenderVm<'m> + Sized
where
    Self::Group: IncrGroupContext<Self>,
{
    #[doc(hidden)]
    /// Default implemenetion to Render an item into the a `<g/>` element.
    fn _render_diff_item(
        &mut self,
        state: RenderState,
        next_abs_ref: &Fingerprint,
        prev_abs_ref: &Fingerprint,
    ) -> Self::Resultant {
        let next_item: &'m ir::VecItem = self.get_item(next_abs_ref).unwrap();
        let prev_item = self.get_item(prev_abs_ref);

        let mut group_ctx = self.start_flat_group(next_abs_ref);

        match &next_item {
            ir::VecItem::Group(group, sz) => {
                let mut group_ctx = group_ctx
                    .with_reuse(self, prev_abs_ref)
                    .with_frame(self, group);
                self.render_diff_group_ref(state, &mut group_ctx, prev_item, group, sz);
                group_ctx
            }
            ir::VecItem::Item(transformed) => {
                let mut group_ctx = group_ctx
                    .with_reuse(self, prev_abs_ref)
                    .transform(self, &transformed.0);
                self.render_diff_transformed_ref(state, &mut group_ctx, prev_item, transformed);
                group_ctx
            }
            ir::VecItem::Text(text) => {
                let group_ctx = group_ctx.with_text(self, text, next_abs_ref, state);
                self.render_diff_flat_text(state, group_ctx, next_abs_ref, prev_abs_ref, text)
            }
            ir::VecItem::Path(path) => {
                group_ctx.render_path(state, self, path, next_abs_ref);
                group_ctx
            }
            ir::VecItem::Link(link) => {
                group_ctx.render_link(self, link);
                group_ctx
            }
            ir::VecItem::Image(image) => {
                group_ctx.render_image(self, image);
                group_ctx
            }
            ir::VecItem::ContentHint(c) => {
                group_ctx.render_content_hint(self, *c);
                group_ctx
            }
            ir::VecItem::Gradient(..) | ir::VecItem::Pattern(..) | ir::VecItem::None => {
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
        prev_item_: Option<&ir::VecItem>,
        next: &ir::GroupRef,
        sz: &Option<Size>,
    ) {
        if let Some(sz) = sz {
            state = state.with_transform(Transform::identity()).with_size(*sz);
        }

        if let Some(ir::VecItem::Group(prev_group, _)) = prev_item_ {
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
        prev_item_: Option<&ir::VecItem>,
        transformed: &ir::TransformedRef,
    ) {
        let child_ref = &transformed.1;
        let state = state.pre_apply(&transformed.0);
        match prev_item_ {
            // if both items are transformed, we can reuse the internal item with transforming it a
            // bit.
            Some(ir::VecItem::Item(ir::TransformedRef(_item, prev_ref))) => {
                ts.render_diff_item_ref_at(state, self, Point::default(), child_ref, prev_ref);
            }
            _ => ts.render_item_ref(state, self, child_ref),
        }
        // failed to reuse
    }

    /// Render a diff text into the underlying context.
    fn render_diff_flat_text(
        &mut self,
        state: RenderState,
        group_ctx: Self::Group,
        next_abs_ref: &Fingerprint,
        _prev_abs_ref: &Fingerprint,
        text: &ir::TextItem,
    ) -> Self::Group {
        self.render_flat_text(state, group_ctx, next_abs_ref, text)
    }
}
