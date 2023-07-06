use std::{ops::Deref, sync::Arc};

use super::ir;
use crate::{
    ir::{AbsoluteRef, Point, Scalar},
    vector::{GroupContext, SvgTextBuilder, SvgTextNode, TransformContext},
    ExportFeature, SvgRenderTask,
};

/// A RAII trait for rendering flatten SVG items into underlying context.
pub trait FlatGroupContext<C>: Sized {
    fn render_item_ref_at(&mut self, ctx: &mut C, pos: Point, item: &AbsoluteRef);
    fn render_item_ref(&mut self, ctx: &mut C, item: &AbsoluteRef) {
        self.render_item_ref_at(ctx, Point::default(), item);
    }

    fn render_glyph_ref(&mut self, ctx: &mut C, pos: Scalar, item: &AbsoluteRef);

    fn render_flat_text_semantics(
        &mut self,
        _ctx: &mut C,
        _text: &ir::FlatTextItem,
        _width: Scalar,
    ) {
    }

    fn with_frame(self, _ctx: &mut C, _group: &ir::GroupRef) -> Self;
    fn with_text(self, ctx: &mut C, text: &ir::FlatTextItem) -> Self;
    fn with_reuse(self, _ctx: &mut C, v: &AbsoluteRef) -> Self;
}

/// A virtual machine for rendering a flatten frame.
/// This is a stateful object that is used to render a frame.
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub trait FlatRenderVm<'m>: Sized {
    type Resultant;
    type Group: GroupContext<Self>
        + FlatGroupContext<Self>
        + TransformContext<Self>
        + Into<Self::Resultant>;

    fn get_item(&self, value: &AbsoluteRef) -> Option<&'m ir::FlatSvgItem>;

    fn start_flat_group(&mut self, value: &AbsoluteRef) -> Self::Group;

    fn start_flat_frame(&mut self, value: &AbsoluteRef, _group: &ir::GroupRef) -> Self::Group {
        self.start_flat_group(value)
    }

    fn start_flat_text(&mut self, value: &AbsoluteRef, _text: &ir::FlatTextItem) -> Self::Group {
        self.start_flat_group(value)
    }

    /// Render an item into the a `<g/>` element.
    fn render_flat_item(&mut self, abs_ref: &AbsoluteRef) -> Self::Resultant {
        let item: &'m ir::FlatSvgItem = self.get_item(abs_ref).unwrap();
        match item.deref() {
            ir::FlatSvgItem::Group(group) => self.render_group_ref(abs_ref, group),
            ir::FlatSvgItem::Item(transformed) => self.render_transformed_ref(abs_ref, transformed),
            ir::FlatSvgItem::Text(text) => self.render_flat_text(abs_ref, text),
            ir::FlatSvgItem::Path(path) => {
                let mut g = self.start_flat_group(abs_ref);
                g.render_path(self, path);
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
            ir::FlatSvgItem::None => {
                panic!("FlatRenderVm.RenderFrame.UnknownItem {:?}", item)
            }
        }
    }

    /// Render a frame group into underlying context.
    fn render_group_ref(&mut self, abs_ref: &AbsoluteRef, group: &ir::GroupRef) -> Self::Resultant {
        let mut group_ctx = self.start_flat_frame(abs_ref, group);

        for (pos, item_ref) in group.0.iter() {
            // let item = self.get_item(&item_ref).unwrap();
            group_ctx.render_item_ref_at(self, *pos, item_ref);
        }

        group_ctx.into()
    }

    /// Render a transformed frame into underlying context.
    fn render_transformed_ref(
        &mut self,
        abs_ref: &AbsoluteRef,
        transformed: &ir::TransformedRef,
    ) -> Self::Resultant {
        let mut ts = self
            .start_flat_group(abs_ref)
            .transform(self, &transformed.0);

        let item_ref = &transformed.1;
        // let item = self.get_item(&item_ref).unwrap();
        ts.render_item_ref(self, item_ref);
        ts.into()
    }

    /// Render a text into the underlying context.
    fn render_flat_text(
        &mut self,
        abs_ref: &AbsoluteRef,
        text: &ir::FlatTextItem,
    ) -> Self::Resultant {
        let group_ctx = self.start_flat_text(abs_ref, text);

        let ppem = Scalar(text.shape.ppem.0);

        let mut group_ctx = group_ctx.transform_scale(self, ppem, -ppem);

        let mut x = 0f32;
        for (offset, advance, glyph) in text.content.glyphs.iter() {
            let offset = x + offset.0;
            let ts = offset / ppem.0;

            group_ctx.render_glyph_ref(self, Scalar(ts), glyph);

            x += advance.0;
        }

        group_ctx.render_flat_text_semantics(self, text, Scalar(x));
        group_ctx.into()
    }
}

impl<'m, 't, Feat: ExportFeature> FlatRenderVm<'m> for SvgRenderTask<'m, 't, Feat> {
    // type Resultant = String;
    type Resultant = Arc<SvgTextNode>;
    type Group = SvgTextBuilder;

    fn get_item(&self, value: &AbsoluteRef) -> Option<&'m ir::FlatSvgItem> {
        self.module.get_item(value)
    }

    fn start_flat_group(&mut self, v: &AbsoluteRef) -> Self::Group {
        Self::Group {
            attributes: vec![("data-tid", v.as_svg_id("g"))],
            content: Vec::with_capacity(1),
        }
    }

    fn start_flat_frame(&mut self, value: &AbsoluteRef, _group: &ir::GroupRef) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.attributes.push(("class", "typst-group".to_owned()));
        g
    }

    fn start_flat_text(&mut self, value: &AbsoluteRef, text: &ir::FlatTextItem) -> Self::Group {
        let mut g = self.start_flat_group(value);
        g.with_text_shape(self, &text.shape);
        g
    }
}
