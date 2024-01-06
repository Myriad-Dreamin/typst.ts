use typst::model::Document;
use typst_ts_core::vector::ir::{
    Abs, FlatModule, ItemPack, LayoutRegion, LayoutRegionNode, ModuleMetadata, MultiVecDocument,
};
use typst_ts_core::vector::pass::Typst2VecPass;

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    pub typst2vec: Typst2VecPass,
    pub layouts: Vec<(Abs, LayoutRegionNode)>,
}

impl DynamicLayoutSvgExporter {
    pub fn render(&mut self, output: &Document) -> LayoutRegionNode {
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

    pub fn debug_stat(&self) -> String {
        let v = self.typst2vec.finalize_ref();
        let item_cnt = v.items.len();
        let glyph_cnt = v.glyphs.len();
        // let glyphs = GlyphPack::from_iter(v.1);

        let module_data = FlatModule::new(vec![
            // ModuleMetadata::Font(Arc::new(fonts)),
            // ModuleMetadata::Glyph(Arc::new(glyphs)),
            ModuleMetadata::Item(ItemPack(v.items.into_iter().collect())),
        ])
        .to_bytes();
        format!(
            "module size: {} bytes, items count: {}, glyph count: {}",
            module_data.len(),
            item_cnt,
            glyph_cnt
        )
    }
}
