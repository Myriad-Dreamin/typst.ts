use reflexo::{
    hash::Fingerprint,
    vector::ir::{Module, Page, Transform},
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
        };
        worker.work(page)
    }
}

pub struct VecIsolatePassWorker<'a> {
    bbox: &'a mut Vec2BBoxPass,
    input: &'a Module,
    output: &'a mut Module,
}

impl<'a> VecIsolatePassWorker<'a> {
    pub fn work(&mut self, page: &Page) -> Page {
        // Analyze stage: determine part to isolate
        self.analyze(page.content, Transform::identity());
        Page {
            // Convert stage: intern and put the content to output
            content: self.item(page.content, Transform::identity()),
            size: page.size,
        }
    }

    pub fn analyze(&mut self, v: Fingerprint, ts: Transform) {
        let bbox = self.bbox.bbox_of(self.input, v, ts);
        let _ = bbox;
        self.output
            .items
            .insert(v, self.input.get_item(&v).unwrap().clone());
    }

    pub fn item(&mut self, v: Fingerprint, ts: Transform) -> Fingerprint {
        let bbox = self.bbox.bbox_of(self.input, v, ts);
        let _ = bbox;
        self.output
            .items
            .insert(v, self.input.get_item(&v).unwrap().clone());
        v
    }
}
