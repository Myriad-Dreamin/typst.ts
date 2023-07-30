use std::sync::Arc;

use typst::doc::Document;
use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        flat_ir::{flatten_glyphs, FlatModule, ItemPack, ModuleBuilder, MultiSvgDocument},
        ir::{Abs, GlyphMapping, Size},
        LowerBuilder,
    },
};

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    builder: ModuleBuilder,
    layouts: Vec<(Abs, Vec<(Fingerprint, Size)>)>,
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
        let item_cnt = v.0.items.len();
        let glyph_cnt = v.1.len();
        let glyphs = flatten_glyphs(v.1);

        let module_data = FlatModule {
            metadata: vec![],
            item_pack: ItemPack(v.0.items.into_iter().collect()),
            glyphs,
            layouts: vec![],
        }
        .to_bytes();
        format!(
            "module size: {} bytes, items count: {}, glyph count: {}",
            module_data.len(),
            item_cnt,
            glyph_cnt
        )
    }
}
