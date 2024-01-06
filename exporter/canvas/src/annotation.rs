use tiny_skia as sk;

use typst_ts_core::{
    annotation::{
        link::{AnnotationBox, LinkAction, UrlOpenAction},
        AnnotationList, LinkAnnotation,
    },
    hash::Fingerprint,
    vector::ir::{self, GroupRef, Module, VecItem},
};

/// Task to create annotation list with vector IR
/// The 'm lifetime is the lifetime of the module which stores the frame data.
pub struct AnnotationListTask<'m, 't> {
    pub module: &'m Module,

    pub page_num: u32,
    pub annotations: &'t mut AnnotationList,
}

// todo: ugly implementation
impl<'m, 't> AnnotationListTask<'m, 't> {
    pub fn new(module: &'m Module, annotations: &'t mut AnnotationList) -> Self {
        Self {
            module,
            page_num: 0,
            annotations,
        }
    }

    pub fn process_flat_item(&mut self, ts: sk::Transform, item: &Fingerprint) {
        let item = self.module.get_item(item).unwrap();
        match item {
            VecItem::Item(t) => self.process_flat_item(
                ts.pre_concat({
                    let t: typst_ts_core::vector::geom::Transform = t.0.clone().into();
                    t.into()
                }),
                &t.1,
            ),
            VecItem::Group(group, _) => self.process_flat_group(ts, group),
            VecItem::Link(link) => self.process_link(ts, link),
            _ => {}
        }
    }

    fn process_flat_group(&mut self, ts: sk::Transform, group: &GroupRef) {
        for (pos, item) in group.0.as_ref() {
            let ts = ts.pre_translate(pos.x.0, pos.y.0);

            self.process_flat_item(ts, item);
        }
    }

    fn process_link(&mut self, ts: sk::Transform, link: &ir::LinkItem) {
        let annotation_box = AnnotationBox {
            page_ref: self.page_num,
            width: link.size.x.0,
            height: link.size.y.0,
            transform: [ts.sx, ts.ky, ts.kx, ts.sy, ts.tx, ts.ty],
        };

        // let action = match dest {
        //     Destination::Url(url) => LinkAction::Url(UrlOpenAction {
        //         url: url.to_string(),
        //     }),
        //     Destination::Position(pos) => LinkAction::GoTo(GoToAction {
        //         page_ref: pos.page.get() as u32,
        //         x: pos.point.x.to_f32(),
        //         y: pos.point.y.to_f32(),
        //     }),
        //     _ => panic!("Unsupported destination type"),
        // };

        self.annotations.links.push(LinkAnnotation {
            annotation_box,
            // todo: goto action
            action: LinkAction::Url(UrlOpenAction {
                url: link.href.as_ref().to_owned(),
            }),
        });
    }
}
