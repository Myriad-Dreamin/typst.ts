use std::sync::Arc;

use reflexo::hash::Fingerprint;
use reflexo::typst::TypstPagedDocument;
use reflexo::vector::{
    ir::{Module, Page, Size, VecDocument, VecItem},
    vm::RenderVm,
};
use reflexo_typst2vec::pass::Typst2VecPass;

use crate::{
    backend::{generate_text, SvgText, SvgTextNode},
    ExportFeature, SvgDataSelection, SvgExporter, SvgTask,
};

impl<Feat: ExportFeature> SvgTask<'_, Feat> {
    /// Render a document into the svg_body.
    pub fn render(&mut self, module: &Module, pages: &[Page], svg_body: &mut Vec<SvgText>) {
        let mut render_task = self.get_render_context(module);

        let mut acc_height = 0u32;
        for page in pages.iter() {
            let entry = &page.content;
            let size = Self::page_size(page.size);

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes: vec![
                    ("class", "typst-page".into()),
                    ("transform", format!("translate(0, {})", acc_height)),
                    ("data-tid", entry.as_svg_id("p")),
                    ("data-page-width", size.x.to_string()),
                    ("data-page-height", size.y.to_string()),
                ],
                content: vec![SvgText::Content(render_task.render_item(entry))],
            })));
            acc_height += size.y;
        }
    }

    pub fn render_patterns(
        &mut self,
        module: &Module,
    ) -> Vec<(Fingerprint, Size, Arc<SvgTextNode>)> {
        self.collect_patterns(|t: &mut Self, id| match module.get_item(id) {
            Some(VecItem::Pattern(g)) => {
                let size = g.size + g.spacing;
                let content = t.get_render_context(module).render_item(&g.frame);
                Some((*id, size, content))
            }
            _ => {
                // #[cfg(debug_assertions)]
                panic!("Invalid pattern reference: {}", id.as_svg_id("p"));
                #[allow(unreachable_code)]
                None
            }
        })
    }
}

impl<Feat: ExportFeature> SvgExporter<Feat> {
    pub fn svg_doc(output: &TypstPagedDocument) -> VecDocument {
        let typst2vec = Typst2VecPass::default();
        let pages = typst2vec.doc(&output.introspector, output);

        let module = typst2vec.finalize();
        VecDocument { pages, module }
    }

    pub fn render_flat_svg(
        module: &Module,
        pages: &[Page],
        parts: Option<SvgDataSelection>,
    ) -> String {
        generate_text(Self::render(module, pages, parts))
    }
}
