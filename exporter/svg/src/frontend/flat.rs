use std::sync::Arc;

use typst::{diag::SourceResult, model::Document};
use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        flat_ir::{
            flatten_glyphs, FlatModule, FlatSvgItem, ItemPack, LayoutRegion, LayoutRegionNode,
            LayoutRegionRepr, Module, ModuleBuilder, ModuleMetadata, Page, SvgDocument,
        },
        flat_vm::FlatRenderVm,
        ir::Size,
        vm::RenderState,
        LowerBuilder,
    },
};

use crate::{
    backend::{generate_text, SvgText, SvgTextNode},
    ExportFeature, SvgDataSelection, SvgExporter, SvgTask,
};

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Render a document into the svg_body.
    pub fn render(&mut self, module: &Module, pages: &[Page], svg_body: &mut Vec<SvgText>) {
        let mut render_task = self.get_render_context(module);

        let mut acc_height = 0u32;
        for page in pages.iter() {
            let entry = &page.content;
            let size = Self::page_size(page.size);

            let state = RenderState::new_size(page.size);
            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes: vec![
                    ("class", "typst-page".into()),
                    ("transform", format!("translate(0, {})", acc_height)),
                    ("data-tid", entry.as_svg_id("p")),
                    ("data-page-width", size.x.to_string()),
                    ("data-page-height", size.y.to_string()),
                ],
                content: vec![SvgText::Content(render_task.render_flat_item(state, entry))],
            })));
            acc_height += size.y;
        }
    }

    pub fn render_flat_patterns(
        &mut self,
        module: &Module,
    ) -> Vec<(Fingerprint, Size, Arc<SvgTextNode>)> {
        self.collect_patterns(|t: &mut Self, id| match module.get_item(id) {
            Some(FlatSvgItem::Pattern(g)) => {
                let size = g.size + g.spacing;
                let state = RenderState::new_size(size);
                let content = t
                    .get_render_context(module)
                    .render_flat_item(state, &g.frame);
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
    pub fn svg_doc(output: &Document) -> SvgDocument {
        let mut lower_builder = LowerBuilder::new(output);
        let mut builder = ModuleBuilder::default();
        let pages = output
            .pages
            .iter()
            .map(|p| {
                let abs_ref = builder.build(lower_builder.lower(p));
                Page {
                    content: abs_ref,
                    size: p.size().into(),
                }
            })
            .collect::<Vec<_>>();
        let module = builder.finalize();
        SvgDocument { pages, module }
    }

    pub fn render_flat_svg(
        module: &Module,
        pages: &[Page],
        parts: Option<SvgDataSelection>,
    ) -> String {
        generate_text(Self::render(module, pages, parts))
    }
}

pub fn export_module(output: SvgDocument) -> SourceResult<Vec<u8>> {
    let SvgDocument { pages, module } = output;
    let glyphs = flatten_glyphs(module.glyphs).into();

    let module_data = FlatModule::new(vec![
        ModuleMetadata::Item(ItemPack(module.items.into_iter().collect())),
        ModuleMetadata::Font(Arc::new(module.fonts.into())),
        ModuleMetadata::Glyph(Arc::new(glyphs)),
        ModuleMetadata::Layout(Arc::new(vec![LayoutRegion::ByScalar(LayoutRegionRepr {
            kind: "width".into(),
            layouts: vec![(
                Default::default(),
                LayoutRegionNode::Pages(Arc::new((Default::default(), pages))),
            )],
        })])),
    ])
    .to_bytes();

    Ok(module_data)
}
