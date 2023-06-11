#![allow(dead_code)]

use typst::{doc::Destination, geom::Size};
use typst_ts_core::{
    annotation::{
        link::{AnnotationBox, GoToAction, LinkAction, UrlOpenAction},
        LinkAnnotation,
    },
    error::prelude::*,
};

use super::SvgRenderTask;
use crate::{sk, utils::AbsExt, RenderFeature};

impl<'m, 't, Feat: RenderFeature> SvgRenderTask<'m, 't, Feat> {
    /// Render a semantic link
    pub(crate) fn render_link(
        &mut self,
        ts: sk::Transform,
        sz: &Size,
        dest: &Destination,
    ) -> ZResult<()> {
        let annotation_box = AnnotationBox {
            page_ref: self.page_off as u32,
            width: sz.x.to_f32(),
            height: sz.y.to_f32(),
            transform: [ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty],
        };

        let action = match dest {
            Destination::Url(url) => LinkAction::Url(UrlOpenAction {
                url: url.to_string(),
            }),
            Destination::Position(pos) => LinkAction::GoTo(GoToAction {
                page_ref: pos.page.get() as u32,
                x: pos.point.x.to_f32(),
                y: pos.point.y.to_f32(),
            }),
            _ => panic!("Unsupported destination type"),
        };

        self.annotations.links.push(LinkAnnotation {
            annotation_box,
            action,
        });
        Ok(())
    }
}
