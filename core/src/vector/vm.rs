use crate::hash::{item_hash128, Fingerprint};

use super::ir::{self, Abs, Axes, Point, Ratio, Scalar, Size, SvgItem, Transform};

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

/// A RAII trait for rendering SVG items into underlying context.
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

    fn begin_text(&mut self, _text: &ir::TextItem) {}

    fn end_text(&mut self, _state: RenderState, _width: Scalar, _text: &ir::TextItem) {}

    /// Render an item at point into underlying context.
    fn render_item_at(&mut self, state: RenderState, ctx: &mut C, pos: Point, item: &SvgItem);
    /// Render an item into underlying context.
    fn render_item(&mut self, state: RenderState, ctx: &mut C, item: &SvgItem) {
        self.render_item_at(state, ctx, Point::default(), item);
    }

    /// Render a semantic text into underlying context.
    fn render_semantic_text(&mut self, _ctx: &mut C, _text: &ir::TextItem, _width: Scalar) {}

    /// Render a glyph into underlying context.
    fn render_glyph(&mut self, _ctx: &mut C, _pos: Scalar, _item: &ir::GlyphItem) {}

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
        Transform::from_scale(self.size.x, self.size.y)
            .post_concat(self.transform.invert().unwrap())
    }

    pub fn at(&self, pos: &Fingerprint) -> Fingerprint {
        // todo: performance
        let item = (*self, *pos);
        Fingerprint::from_u128(item_hash128(&item))
    }
}

/// A trait for rendering SVG items into underlying context.
pub trait RenderVm: Sized {
    type Resultant;
    type Group: GroupContext<Self> + TransformContext<Self> + Into<Self::Resultant>;

    /// Start a new `<g/>` like object.
    fn start_group(&mut self) -> Self::Group;

    /// Start a new `<g/>` like object for frame group.
    fn start_frame(&mut self, _group: &ir::GroupItem) -> Self::Group {
        self.start_group()
    }

    /// Start a new `<g/>` like object for text.
    fn start_text(&mut self, _state: RenderState, _text: &ir::TextItem) -> Self::Group {
        self.start_group()
    }

    /// Render an item into underlying context.
    fn render_item(&mut self, state: RenderState, item: &SvgItem) -> Self::Resultant {
        match &item {
            ir::SvgItem::Group(group, sz) => self.render_group(state, group, sz),
            ir::SvgItem::Transformed(transformed) => self.render_transformed(state, transformed),
            ir::SvgItem::Text(text) => self.render_text(state, text),
            ir::SvgItem::Path((path, ..)) => {
                let mut g = self.start_group();
                g.render_path(
                    state,
                    self,
                    path,
                    &Fingerprint::from_u128(item_hash128(path)),
                );
                g.into()
            }
            ir::SvgItem::Link(link) => {
                let mut g = self.start_group();
                g.render_link(self, link);
                g.into()
            }
            ir::SvgItem::Image((image, ..)) => {
                let mut g = self.start_group();
                g.render_image(self, image);
                g.into()
            }
            ir::SvgItem::Gradient(..) => {
                panic!("RenderVm.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render a frame group into underlying context.
    fn render_group(
        &mut self,
        mut state: RenderState,
        group: &ir::GroupItem,
        sz: &Option<Size>,
    ) -> Self::Resultant {
        let mut group_ctx = self.start_frame(group);

        if let Some(sz) = sz {
            state = state.with_transform(Transform::identity()).with_size(*sz);
        }

        for (pos, item_ref) in group.0.iter() {
            group_ctx.render_item_at(state.pre_translate(*pos), self, *pos, item_ref);
        }

        group_ctx.into()
    }

    /// Render a transformed frame into underlying context.
    fn render_transformed(
        &mut self,
        state: RenderState,
        transformed: &ir::TransformedItem,
    ) -> Self::Resultant {
        let mut ts = self.start_group().transform(self, &transformed.0);
        ts.render_item(state.pre_apply(&transformed.0), self, &transformed.1);
        ts.into()
    }

    /// Render a text into the underlying context.
    // todo: combine with flat item one
    fn render_text(&mut self, state: RenderState, text: &ir::TextItem) -> Self::Resultant {
        let group_ctx = self.start_text(state, text);

        // upem is the unit per em defined in the font.
        // ppem is calcuated by the font size.
        // > ppem = text_size / upem
        let upem = text.font.units_per_em() as f32;
        let ppem = Scalar(text.shape.size.0 / upem);
        let inv_ppem = upem / text.shape.size.0;

        let mut group_ctx = group_ctx.transform_scale(self, ppem, -ppem);

        group_ctx.begin_text(text);
        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.0;
            let ts = offset * inv_ppem;

            group_ctx.render_glyph(self, Scalar(ts), glyph);

            x += advance.0;
        }
        group_ctx.end_text(state, Scalar(x), text);

        group_ctx.render_semantic_text(self, text, Scalar(x));
        group_ctx.into()
    }
}
