use crate::hash::{item_hash128, Fingerprint};

use super::ir::{self, Abs, Axes, Point, Ratio, Scalar, Size, Transform};

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

    /// Render a semantic link into underlying context.
    fn render_content_hint(&mut self, _ctx: &mut C, _ch: char) {}
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
}
