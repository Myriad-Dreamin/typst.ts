use std::sync::Arc;

use typst::{diag::SourceResult, doc::Document};
use typst_ts_core::vector::{
    flat_ir::{self, Module, ModuleBuilder, Pages, SvgDocument},
    flat_vm::FlatRenderVm,
    ir::GlyphMapping,
    LowerBuilder,
};

use crate::{
    backend::{generate_text, SvgText, SvgTextNode},
    ExportFeature, SvgExporter, SvgTask,
};

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Render a document into the svg_body.
    pub fn render(&mut self, module: &Module, pages: &Pages, svg_body: &mut Vec<SvgText>) {
        let mut render_task = self.get_render_context(module);

        let mut acc_height = 0u32;
        for page in pages.iter() {
            let entry = &page.0;
            let size = Self::page_size(page.1);

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes: vec![
                    ("transform", format!("translate(0, {})", acc_height)),
                    ("data-tid", entry.as_svg_id("p")),
                    ("data-page-width", size.x.to_string()),
                    ("data-page-height", size.y.to_string()),
                ],
                content: vec![SvgText::Content(render_task.render_flat_item(entry))],
            })));
            acc_height += size.y;
        }
    }
}

impl<Feat: ExportFeature> SvgExporter<Feat> {
    pub(crate) fn header(output: &Pages) -> String {
        // calculate the width and height of the svg
        let w = output
            .iter()
            .map(|p| p.1.x.0.ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output.iter().map(|p| p.1.y.0.ceil()).sum::<f32>();

        Self::header_inner(w, h)
    }

    pub fn svg_doc(output: &Document) -> (SvgDocument, GlyphMapping) {
        let mut lower_builder = LowerBuilder::new(output);
        let mut builder = ModuleBuilder::default();
        let pages = output
            .pages
            .iter()
            .map(|p| {
                let abs_ref = builder.build(lower_builder.lower(p));
                (abs_ref.fingerprint, p.size().into())
            })
            .collect::<Vec<_>>();
        let (module, glyph_mapping) = builder.finalize();

        (SvgDocument { pages, module }, glyph_mapping)
    }

    pub fn render_flat_svg(module: &Module, pages: &Pages) -> String {
        let header = Self::header(pages);

        let mut t = SvgTask::<Feat>::default();
        let mut svg_body = vec![];
        t.render(module, pages, &mut svg_body);

        let glyphs = t.render_glyphs(module.glyphs.iter().map(|(x, y)| (x, y)), true);

        generate_text(Self::render_svg_template(
            t,
            header,
            svg_body,
            glyphs.into_iter(),
        ))
    }
}

pub fn export_module(output: &Document) -> SourceResult<Vec<u8>> {
    let mut t = LowerBuilder::new(output);

    let mut builder = ModuleBuilder::default();

    for page in output.pages.iter() {
        let item = t.lower(page);
        let _entry_id = builder.build(item);
    }

    let (repr, ..) = builder.finalize();

    Ok(flat_ir::serialize_module(repr))
}
