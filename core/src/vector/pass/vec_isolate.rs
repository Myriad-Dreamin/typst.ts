//! Private discussion: https://typst.app/project/pl4GDUVJT4_yVhEhiYrwle

#![allow(dead_code)]

use std::collections::HashMap;

use reflexo::{
    hash::Fingerprint,
    vector::ir::{Module, Page, Rect, Transform, TransformItem, VecItem},
};

use super::Vec2BBoxPass;

#[derive(Default)]
pub struct VecIsolatePass {
    bbox: Vec2BBoxPass,
    output: Module,
}

impl VecIsolatePass {
    pub fn page(&mut self, input: &Module, page: &Page) -> Page {
        let mut worker = VecIsolatePassWorker {
            bbox: &mut self.bbox,
            input,
            output: &mut self.output,
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
    items: Vec<Fingerprint>,
}

pub struct VecIsolatePassWorker<'a> {
    bbox: &'a mut Vec2BBoxPass,
    input: &'a Module,
    output: &'a mut Module,
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
            self.analyze1(v, Transform::identity());

            // Take state
            let last_bbox = self.current_bbox.take();
            if let Some(last_group) = self.groups.last_mut() {
                last_group.bbox = last_bbox.unwrap_or_default();
            }
            let group = std::mem::take(&mut self.groups);

            self.analyze2(group);
        }
    }

    // Algorithm L_{1,1}
    fn analyze1(&mut self, v: Fingerprint, ts: Transform) {
        let item = self.input.get_item(&v).unwrap();
        match item {
            VecItem::Group(g) => {
                for (p, v) in g.0.iter() {
                    let ts = ts.pre_translate(p.x.0, p.y.0);
                    self.analyze1(*v, ts);
                }
            }
            VecItem::Item(it) => {
                // Either ignore the transform,
                if it.0.is_identity() {
                    self.analyze1(it.1, ts);
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

                    self.analyze1_leaf(it.1, ts, clip_box.as_ref());
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
                self.analyze1_leaf(v, ts, None);
            }
        }
    }

    fn analyze1_leaf(&mut self, itv: Fingerprint, ts: Transform, clip_box: Option<&Rect>) {
        let it = self.input.get_item(&itv).unwrap().clone();
        self.output.items.insert(itv, it);

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
                        items: vec![itv],
                    });
                } else {
                    *current_bbox = bbox.union(current_bbox);
                }
            }
            current_bbox => {
                self.groups.push(GroupBox {
                    bbox,
                    items: vec![itv],
                });
                *current_bbox = Some(bbox);
            }
        }
    }

    // Algorithm L_{1,2}
    fn analyze2(&self, groups: Vec<GroupBox>) {
        let _ = groups;
        let _ = self.remapped;
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
