use reflexo::typst::TypstPagedDocument;
use reflexo_typst2vec::ir::{Abs, LayoutRegion, LayoutRegionNode, MultiVecDocument};
use reflexo_typst2vec::pass::Typst2VecPass;

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    pub typst2vec: Typst2VecPass,
    pub layouts: Vec<(Abs, LayoutRegionNode)>,
}

impl DynamicLayoutSvgExporter {
    pub fn render(&mut self, output: &TypstPagedDocument) -> LayoutRegionNode {
        self.typst2vec.reset();
        // let instant = std::time::Instant::now();
        // check the document
        // let mut t = LowerBuilder::new(output);

        let pages = self.typst2vec.doc(&output.introspector, output);

        // log::trace!("svg dynamic layout render time: {:?}",
        // instant.elapsed());

        LayoutRegionNode::new_pages(pages)
    }

    pub fn finalize(self) -> MultiVecDocument {
        let module = self.typst2vec.finalize();
        MultiVecDocument {
            module,
            layouts: vec![LayoutRegion::new_by_scalar("width".into(), self.layouts)],
        }
    }
}
