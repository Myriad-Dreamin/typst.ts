//! Rendering into web_sys::CanvasRenderingContext2d.

use std::sync::Arc;

use ir::{GroupItem, PathItem, SvgItem, TransformItem};

use crate::svg::SvgPath2DBuilder;
use crate::utils::AbsExt;
use crate::{ir, RenderFeature, SvgRenderTask};
use typst::doc::{Frame, FrameItem, GroupItem as TypstGroupItem};
use typst_ts_core::error::prelude::*;

pub(crate) mod image;
pub(crate) mod shape;
pub(crate) mod text;

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Lower a frame into svg item.
    pub fn lower(&mut self, frame: &Frame) -> ZResult<SvgItem> {
        self.lower_frame(frame)
    }

    /// Lower a frame into svg item.
    fn lower_frame(&mut self, frame: &Frame) -> ZResult<SvgItem> {
        let mut items = vec![];

        for (pos, item) in frame.items() {
            let item = match item {
                FrameItem::Group(group) => self.lower_group(group)?,
                FrameItem::Text(text) => self.lower_text(text),
                FrameItem::Shape(shape, _) => self.lower_shape(shape)?,
                FrameItem::Image(image, size, _) => self.lower_image(image, *size),
                FrameItem::Meta(..) => continue,
            };

            items.push((*pos, item));
        }

        Ok(SvgItem::Group(Arc::new(GroupItem(items.into()))))
    }

    /// Lower a group frame with optional transform and clipping into svg item.
    fn lower_group(&mut self, group: &TypstGroupItem) -> ZResult<SvgItem> {
        let mut inner = self.lower_frame(&group.frame)?;

        println!(
            "group.clips = {} {:#?} {:?}",
            group.clips,
            group.frame.size(),
            group.transform
        );
        if group.clips {
            let mask_box = {
                let mut builder = SvgPath2DBuilder::default();

                // build a rectangle path
                let size = group.frame.size();
                let w = size.x.to_f32();
                let h = size.y.to_f32();
                builder.rect(0., 0., w, h);

                builder.0
            };

            inner = SvgItem::Transformed(Arc::new(ir::TransformedItem(
                TransformItem::Clip(Arc::new(PathItem {
                    d: mask_box,
                    styles: vec![],
                })),
                inner,
            )));
        };

        Ok(SvgItem::Transformed(Arc::new(ir::TransformedItem(
            TransformItem::Matrix(Arc::new(group.transform)),
            inner,
        ))))
    }
}
