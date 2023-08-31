use std::sync::Arc;

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
    builder: ModuleBuilder,
    layouts: Vec<(Abs, LayoutRegionNode)>,
}

impl DynamicLayoutSvgExporter {
    pub fn render(&mut self, layout_width: typst::geom::Abs, output: Arc<Document>) {
        self.builder.reset();
        let instant = std::time::Instant::now();
        // check the document
        let mut t = LowerBuilder::new(&output);

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

        self.layouts
            .push((layout_width.into(), LayoutRegionNode::new_pages(pages)));
        log::trace!("svg dynamic layout render time: {:?}", instant.elapsed());
    }

    pub fn finalize(self) -> MultiSvgDocument {
        let module = self.builder.finalize();
        MultiSvgDocument {
            module,
            layouts: LayoutRegion::by_scalar("width".into(), self.layouts),
        }
    }

    pub fn debug_stat(&self) -> String {
        let v = self.builder.finalize_ref();
        let item_cnt = v.items.len();
        let glyph_cnt = v.glyphs.len();
        // let glyphs = GlyphPack::from_iter(v.1);

        let module_data = FlatModule::new(vec![
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
