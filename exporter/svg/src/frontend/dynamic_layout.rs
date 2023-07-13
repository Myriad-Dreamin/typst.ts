use std::sync::Arc;

use typst::doc::Document;
use typst_ts_core::vector::{
    flat_ir::{ModuleBuilder, MultiSvgDocument},
    ir::{Abs, AbsoluteRef, GlyphMapping, Size},
    LowerBuilder,
};

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    builder: ModuleBuilder,
    layouts: Vec<(Abs, Vec<(AbsoluteRef, Size)>)>,
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
                (abs_ref, p.size().into())
            })
            .collect::<Vec<_>>();

        self.layouts.push((layout_width.into(), pages));
        println!("svg dynamic layout render time: {:?}", instant.elapsed());
    }

    pub fn finalize(self) -> (MultiSvgDocument, GlyphMapping) {
        let (module, glyph_mapping) = self.builder.finalize();
        (
            MultiSvgDocument {
                module,
                layouts: self.layouts,
            },
            glyph_mapping,
        )
    }

    pub fn debug_stat(&self) -> String {
        let v = self.builder.finalize_ref();
        let item_cnt = v.0.item_pack.0.len();
        let glyph_cnt = v.1.len();
        let module_data = crate::flat_ir::serialize_module(v.0);
        format!(
            "module size: {} bytes, items count: {}, glyph count: {}",
            module_data.len(),
            item_cnt,
            glyph_cnt
        )
    }
}
