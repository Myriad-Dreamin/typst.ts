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
    TakeAs,
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
    pub(crate) fn header(output: &[Page]) -> String {
        // calculate the width and height of the svg
        let w = output
            .iter()
            .map(|p| p.size.x.0.ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output.iter().map(|p| p.size.y.0.ceil()).sum::<f32>();

        Self::header_inner(w, h)
    }

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
        let header = Self::header(pages);

        let mut t = SvgTask::<Feat>::default();
        let mut svg_body = vec![];
        t.render(module, pages, &mut svg_body);
        let patterns = t.render_flat_patterns(module);

        let glyphs = t.render_glyphs(
            module.glyphs.iter().enumerate().map(|(x, (_, y))| (x, y)),
            true,
        );

        let gradients = std::mem::take(&mut t.gradients);
        let gradients = gradients
            .values()
            .filter_map(|(_, id, _)| match module.get_item(id) {
                Some(FlatSvgItem::Gradient(g)) => Some((id, g.as_ref())),
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid gradient reference: {}", id.as_svg_id("g"));
                    #[allow(unreachable_code)]
                    None
                }
            });

        generate_text(Self::render_svg_template(
            t,
            header,
            svg_body,
            glyphs,
            gradients,
            patterns.into_iter(),
            parts,
        ))
    }
}

pub fn export_module(output: &Document) -> SourceResult<Vec<u8>> {
    let mut t = LowerBuilder::new(output);

    let mut builder = ModuleBuilder::default();

    let mut pages = vec![];
    for page in output.pages.iter() {
        let item = t.lower(page);
        let content = builder.build(item);
        pages.push(Page {
            content,
            size: page.size().into(),
        });
    }

    for ext in t.extra_items.into_values() {
        builder.build(ext.take());
    }

    let repr: Module = builder.finalize();

    let glyphs = flatten_glyphs(repr.glyphs).into();

    let module_data = FlatModule::new(vec![
        ModuleMetadata::Item(ItemPack(repr.items.into_iter().collect())),
        ModuleMetadata::Font(Arc::new(repr.fonts.into())),
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
