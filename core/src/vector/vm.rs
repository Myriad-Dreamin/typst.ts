use std::ops::Deref;

use super::ir::{self, Abs, Axes, Point, Ratio, Scalar, SvgItem};

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
    fn with_text_shape(&mut self, _ctx: &mut C, _shape: &ir::TextShape) {}

    /// Render an item at point into underlying context.
    fn render_item_at(&mut self, ctx: &mut C, pos: Point, item: &SvgItem);
    /// Render an item into underlying context.
    fn render_item(&mut self, ctx: &mut C, item: &SvgItem) {
        self.render_item_at(ctx, Point::default(), item);
    }

    /// Render a semantic text into underlying context.
    fn render_semantic_text(&mut self, _ctx: &mut C, _text: &ir::TextItem, _width: Scalar) {}

    /// Render a glyph into underlying context.
    fn render_glyph(&mut self, _ctx: &mut C, _pos: Scalar, _item: &ir::GlyphItem) {}

    /// Render a geometrical shape into underlying context.
    fn render_path(&mut self, _ctx: &mut C, _path: &ir::PathItem) {}

    /// Render a semantic link into underlying context.
    fn render_link(&mut self, _ctx: &mut C, _link: &ir::LinkItem) {}

    /// Render an image into underlying context.
    fn render_image(&mut self, _ctx: &mut C, _image_item: &ir::ImageItem) {}

    fn attach_debug_info(&mut self, _ctx: &mut C, _span_id: u64) {}
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
    fn start_text(&mut self, _text: &ir::TextItem) -> Self::Group {
        self.start_group()
    }

    /// Render an item into underlying context.
    fn render_item(&mut self, item: &SvgItem) -> Self::Resultant {
        match item.deref() {
            ir::SvgItem::Group(group) => self.render_group(group),
            ir::SvgItem::Transformed(transformed) => self.render_transformed(transformed),
            ir::SvgItem::Text(text) => self.render_text(text),
            ir::SvgItem::Path((path, ..)) => {
                let mut g = self.start_group();
                g.render_path(self, path);
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
        }
    }

    /// Render a frame group into underlying context.
    fn render_group(&mut self, group: &ir::GroupItem) -> Self::Resultant {
        let mut group_ctx = self.start_frame(group);

        for (pos, item_ref) in group.0.iter() {
            group_ctx.render_item_at(self, *pos, item_ref);
        }

        group_ctx.into()
    }

    /// Render a transformed frame into underlying context.
    fn render_transformed(&mut self, transformed: &ir::TransformedItem) -> Self::Resultant {
        let mut ts = self.start_group().transform(self, &transformed.0);
        ts.render_item(self, &transformed.1);
        ts.into()
    }

    /// Render a text into the underlying context.
    // todo: combine with flat item one
    fn render_text(&mut self, text: &ir::TextItem) -> Self::Resultant {
        let group_ctx = self.start_text(text);

        let ppem = Scalar(text.shape.ppem.0);

        let mut group_ctx = group_ctx.transform_scale(self, ppem, -ppem);

        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.0;
            let ts = offset / ppem.0;

            group_ctx.render_glyph(self, Scalar(ts), glyph);

            x += advance.0;
        }

        group_ctx.render_semantic_text(self, text, Scalar(x));
        group_ctx.into()
    }
}
