use typst::doc::Document;
use typst_ts_core::vector::{
    flat_ir::{
        FlatModule, ItemPack, LayoutRegion, LayoutRegionNode, ModuleBuilder, ModuleMetadata,
        MultiSvgDocument, Page,
    },
    ir::Abs,
    LowerBuilder,
};

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    pub builder: ModuleBuilder,
    pub layouts: Vec<(Abs, LayoutRegionNode)>,
}

impl DynamicLayoutSvgExporter {
    pub fn render(&mut self, output: &Document) -> LayoutRegionNode {
        self.builder.reset();
        // let instant = std::time::Instant::now();
        // check the document
        let mut t = LowerBuilder::new(output);

        let pages = output
            .pages
            .iter()
            .map(|p| {
                let abs_ref = self.builder.build(t.lower(p));
                Page {
                    content: abs_ref,
                    size: p.size().into(),
                }
            })
            .collect::<Vec<_>>();

        for ext in t.extra_items.into_values() {
            self.builder.build(ext);
        }

        // log::trace!("svg dynamic layout render time: {:?}",
        // instant.elapsed());

        LayoutRegionNode::new_pages(pages)
    }

    pub fn finalize(self) -> MultiSvgDocument {
        let module = self.builder.finalize();
        MultiSvgDocument {
            module,
            layouts: vec![LayoutRegion::new_by_scalar("width".into(), self.layouts)],
        }
    }

    pub fn debug_stat(&self) -> String {
        let v = self.builder.finalize_ref();
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
