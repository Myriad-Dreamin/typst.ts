//! Private discussion: https://typst.app/project/pl4GDUVJT4_yVhEhiYrwle
//! Note: this is a work in progress and not yet finished.

#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use reflexo::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::ir::{GroupRef, Module, Page, Point, Rect, Transform, TransformItem, VecItem},
};

use super::Vec2BBoxPass;

pub enum IsolatedVecItem {}

#[derive(Default)]
pub struct VecIsolatePass {
    bbox: Vec2BBoxPass,
    output: Module,

    fingerprint_builder: FingerprintBuilder,
}

impl VecIsolatePass {
    pub fn page(&mut self, input: &Module, page: &Page) -> Page {
        let mut worker = VecIsolatePassWorker {
            bbox: &mut self.bbox,
            input,
            output: &mut self.output,
            fingerprint_builder: &mut self.fingerprint_builder,
            worklist: Vec::default(),
            remapped: HashMap::default(),

            groups: Vec::default(),
            current_bbox: None,
        };
        worker.work(page)
    }
}

struct GroupBox {
    bbox: Rect,
    items: Vec<(Point, Fingerprint)>,
}

pub struct VecIsolatePassWorker<'a> {
    bbox: &'a mut Vec2BBoxPass,
    input: &'a Module,
    output: &'a mut Module,
    fingerprint_builder: &'a mut FingerprintBuilder,
    worklist: Vec<Fingerprint>,
    remapped: HashMap<Fingerprint, Fingerprint>,

    groups: Vec<GroupBox>,
    current_bbox: Option<Rect>,
}

impl<'a> VecIsolatePassWorker<'a> {
    pub fn work(&mut self, page: &Page) -> Page {
        // Analyze stage: determine part to isolate
        self.worklist.push(page.content);
        self.schedule1();
        Page {
            // Convert stage: intern and put the content to output
            content: self.convert(page.content, Transform::identity()),
            size: page.size,
        }
    }

    fn schedule1(&mut self) {
        while let Some(v) = self.worklist.pop() {
            self.analyze1(v, Point::default(), Transform::identity());

            // Take state
            let last_bbox = self.current_bbox.take();
            if let Some(last_group) = self.groups.last_mut() {
                last_group.bbox = last_bbox.unwrap_or_default();
            }
            let group = std::mem::take(&mut self.groups);

            let _ = self.remapped;
            self.analyze2(group);
        }
    }

    // Algorithm L_{1,1}
    fn analyze1(&mut self, v: Fingerprint, pt: Point, ts: Transform) {
        let item = self.input.get_item(&v).unwrap();
        match item {
            VecItem::Group(g) => {
                for (p, v) in g.0.iter() {
                    self.analyze1(*v, pt + *p, ts);
                }
            }
            VecItem::Item(it) => {
                // Either ignore the transform,
                if it.0.is_identity() {
                    self.analyze1(it.1, pt, ts);
                // or view a transformed item as the leaf
                } else {
                    // Add to the worklist first
                    self.worklist.push(it.1);

                    // todo: this causes larger bbox to calculate clip before leaf analysis
                    let clip_box = if let TransformItem::Clip(c) = &it.0 {
                        self.bbox.path_bbox(c, ts)
                    } else {
                        None
                    };

                    self.analyze1_leaf(it.1, pt, ts, clip_box.as_ref());
                }
            }
            VecItem::None
            | VecItem::Image(_)
            | VecItem::Link(_)
            | VecItem::Path(_)
            | VecItem::Text(_)
            | VecItem::Color32(_)
            | VecItem::Gradient(_)
            | VecItem::Pattern(_)
            | VecItem::ContentHint(_)
            | VecItem::ColorTransform(_) => {
                // todo: page bbox
                self.analyze1_leaf(v, pt, ts, None);
            }
        }
    }

    fn analyze1_leaf(
        &mut self,
        itv: Fingerprint,
        pt: Point,
        ts: Transform,
        clip_box: Option<&Rect>,
    ) {
        let it = self.input.get_item(&itv).unwrap().clone();
        self.output.items.insert(itv, it);

        let ts = ts.pre_translate(pt.x.0, pt.y.0);
        let bbox = self.bbox.bbox_of(self.input, itv, ts);

        let Some(bbox) = bbox.map(|bbox| {
            if let Some(clip_box) = clip_box {
                return bbox.intersect(clip_box);
            }

            bbox
        }) else {
            log::warn!("Failed to calculate bbox for {itv:?}");
            return;
        };

        match &mut self.current_bbox {
            Some(current_bbox) => {
                if bbox.intersect(current_bbox).is_empty() {
                    let bbox = std::mem::replace(current_bbox, bbox);
                    self.groups.last_mut().unwrap().bbox = bbox;
                    self.groups.push(GroupBox {
                        bbox,
                        items: vec![(pt, itv)],
                    });
                } else {
                    *current_bbox = bbox.union(current_bbox);
                }
            }
            current_bbox => {
                self.groups.push(GroupBox {
                    bbox,
                    items: vec![(pt, itv)],
                });
                *current_bbox = Some(bbox);
            }
        }
    }

    // Algorithm L_{1,2}
    fn analyze2(&mut self, inputs: Vec<GroupBox>) {
        if inputs.is_empty() {
            return;
        }

        // The cursor maintains the upper side of the convex hull of the page
        let mut cursors = Vec::default();
        let mut yline = f32::MIN;
        cursors.push((0, inputs.first().unwrap()));
        for (i, c) in inputs.iter().enumerate() {
            let y = c.bbox.lo.y.0;
            if y < yline {
                cursors.push((i, c));
            }
            yline = y;
        }

        // Advance the cursor to the lower side
        // let items = Vec::default();

        // self.store(VecItem::Group(GroupRef(items.into())))
    }

    pub fn convert(&mut self, v: Fingerprint, ts: Transform) -> Fingerprint {
        let bbox = self.bbox.bbox_of(self.input, v, ts);
        println!("Isolating {v:?} with bbox {bbox:?}");
        self.output
            .items
            .insert(v, self.input.get_item(&v).unwrap().clone());
        v
    }
}
