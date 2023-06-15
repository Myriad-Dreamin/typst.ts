use std::ops::Deref;

use crate::{
    ir,
    ir::{Abs, AbsoulteRef, Axes, Point, Ratio, Scalar},
};

pub trait GroupContext: Sized {
    fn transform_matrix(self, matrix: &ir::Transform) -> Self;
    fn transform_translate(self, matrix: Axes<Abs>) -> Self;
    fn transform_scale(self, x: Ratio, y: Ratio) -> Self;
    fn transform_rotate(self, matrix: Scalar) -> Self;
    fn transform_skew(self, matrix: (Ratio, Ratio)) -> Self;
    fn transform_clip(self, matrix: &ir::PathItem) -> Self;

    fn transform(self, transform: &ir::TransformItem) -> Self {
        match transform {
            ir::TransformItem::Matrix(transform) => self.transform_matrix(transform.as_ref()),
            ir::TransformItem::Translate(transform) => self.transform_translate(*transform.clone()),
            ir::TransformItem::Scale(transform) => self.transform_scale(transform.0, transform.1),
            ir::TransformItem::Rotate(transform) => self.transform_rotate(*transform.clone()),
            ir::TransformItem::Skew(transform) => self.transform_skew(*transform.clone()),
            ir::TransformItem::Clip(transform) => self.transform_clip(transform.as_ref()),
        }
    }

    fn drop_item(&mut self, item: AbsoulteRef) {
        self.drop_item_at(Point::default(), item);
    }
    fn drop_item_at(&mut self, pos: Point, item: AbsoulteRef);
    fn drop_glyph(&mut self, pos: Scalar, item: &AbsoulteRef);
}

/// A virtual machine for rendering a frame.
/// This is a stateful object that is used to render a frame.
/// The 's lifetime is the lifetime of the virtual machine itself.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait RenderVm<'s, 'm> {
    type Resultant;
    type Group: GroupContext + Into<Self::Resultant>;

    fn get_item(&self, value: &AbsoulteRef) -> Option<&'m ir::FlatSvgItem>;

    fn start_group(&'s mut self, value: &AbsoulteRef) -> Self::Group;

    fn start_frame(&'s mut self, value: &AbsoulteRef, _group: &ir::GroupRef) -> Self::Group {
        self.start_group(value)
    }

    fn start_text(&'s mut self, value: &AbsoulteRef, _text: &ir::FlatTextItem) -> Self::Group {
        self.start_group(value)
    }

    /// Render an item into the a `<g/>` element.
    fn render_item(&'s mut self, abs_ref: AbsoulteRef) -> Self::Resultant {
        let item: &'m ir::FlatSvgItem = self.get_item(&abs_ref).unwrap();
        self.render_item_inner(abs_ref, item)
    }

    fn render_item_inner(
        &'s mut self,
        abs_ref: AbsoulteRef,
        item: &'m ir::FlatSvgItem,
    ) -> Self::Resultant {
        match item.deref() {
            ir::FlatSvgItem::Group(group) => self.render_group(abs_ref, group),
            ir::FlatSvgItem::Item(transformed) => self.render_transformed(abs_ref, transformed),
            ir::FlatSvgItem::Text(text) => self.render_text(abs_ref, text),
            ir::FlatSvgItem::Path(path) => self.render_path(abs_ref, path),
            ir::FlatSvgItem::Link(link) => self.render_link(abs_ref, link),
            ir::FlatSvgItem::Image(image) => self.render_image(abs_ref, image),
            ir::FlatSvgItem::None => {
                panic!("RenderVm.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render a frame into svg text.
    fn render_group(&'s mut self, abs_ref: AbsoulteRef, group: &ir::GroupRef) -> Self::Resultant {
        let mut group_ctx = self.start_frame(&abs_ref, group);

        for (pos, item) in group.0.iter() {
            let abs_ref = abs_ref.id.make_absolute_ref(item.clone());
            // let item = self.get_item(&def_id).unwrap();

            group_ctx.drop_item_at(*pos, abs_ref);
        }

        group_ctx.into()
    }

    /// Render a transformed frame into svg text.
    fn render_transformed(
        &'s mut self,
        abs_ref: AbsoulteRef,
        transformed: &ir::TransformedRef,
    ) -> Self::Resultant {
        let mut ts = self.start_group(&abs_ref).transform(&transformed.0);

        let item_ref = abs_ref.id.make_absolute_ref(transformed.1.clone());
        // let item = self.get_item(&item_ref).unwrap();

        ts.drop_item(item_ref);
        ts.into()
    }

    /// Render a text run into the svg text.
    fn render_text(&'s mut self, abs_ref: AbsoulteRef, text: &ir::FlatTextItem) -> Self::Resultant {
        let group_ctx = self.start_text(&abs_ref, text);

        let ppem = Scalar(text.shape.ppem.0);

        let mut group_ctx = group_ctx.transform_scale(ppem, -ppem);

        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.0;
            let ts = offset / ppem.0;

            group_ctx.drop_glyph(Scalar(ts), glyph);

            x += advance.0;
        }

        group_ctx.into()
    }

    /// Render a geometrical shape into svg text.
    fn render_path(&'s mut self, abs_ref: AbsoulteRef, path: &ir::PathItem) -> Self::Resultant;

    /// Render a semantic link into svg text.
    fn render_link(&'s mut self, abs_ref: AbsoulteRef, link: &ir::LinkItem) -> Self::Resultant;

    fn render_image(
        &'s mut self,
        abs_ref: AbsoulteRef,
        image_item: &ir::ImageItem,
    ) -> Self::Resultant;
}
