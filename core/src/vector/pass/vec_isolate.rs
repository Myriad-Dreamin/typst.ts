//! Private discussion: https://typst.app/project/pl4GDUVJT4_yVhEhiYrwle

#![allow(dead_code)]

use std::collections::HashMap;

use reflexo::{
    hash::Fingerprint,
    vector::ir::{Module, Page, Rect, Transform, TransformedRef, VecItem},
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
            let groups = self.analyze1(v, Transform::identity());
            self.analyze2(groups);
        }
    }

    // Algorithm L_{1,1}
    fn analyze1(&mut self, v: Fingerprint, ts: Transform) -> Vec<GroupBox> {
        let groups = Vec::default();
        // 这里要一个state
        // State(current_bbox)

        // 这里要一个visitor
        // visitor(|| {

        if let VecItem::Item(it) = self.input.get_item(&v).unwrap() {
            // Identity优化
            fn is_identity(it: &TransformedRef) -> bool {
                let _ = it;
                false
            }
            let is_identity = is_identity(it);
            let _ = is_identity;

            // transformed item就像不透明的盒子
            if is_identity {
                let _ = self.remapped;
                // self.analyze1(it.1);
            } else {
                // self.worklist.push(it.1);
            }
        } else {
            let bbox = self.bbox.bbox_of(self.input, v, ts);
            let _ = bbox;
            self.output
                .items
                .insert(v, self.input.get_item(&v).unwrap().clone());
        }

        //   if bbox.intersect(current_bbox).is_empty() { current_bbox = bbox }
        //     else { current_bbox = current_bbox.union(bbox); }
        let _ = groups;

        // })

        groups
    }

    // Algorithm L_{2,1}
    fn analyze2(&self, groups: Vec<GroupBox>) {
        let _ = groups;
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
