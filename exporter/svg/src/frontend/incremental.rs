use std::{
    collections::{hash_map::RandomState, HashSet},
    ops::Deref,
    sync::Arc,
};

use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        flat_ir::{FlatModule, LayoutRegionNode, Module, ModuleBuilder, MultiSvgDocument, Page},
        flat_vm::{FlatIncrRenderVm, FlatRenderVm},
        incr::{IncrDocClient, IncrDocClientKern, IncrDocServer},
        ir::{Rect, SvgItem},
    },
};

use crate::{
    backend::{SvgText, SvgTextNode},
    ExportFeature, SvgExporter, SvgTask,
};

/// The feature set which is used for exporting incremental rendered svg.
struct IncrementalExportFeature;

impl ExportFeature for IncrementalExportFeature {
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const WITH_BUILTIN_CSS: bool = false;
    const WITH_RESPONSIVE_JS: bool = false;
    const AWARE_HTML_ENTITY: bool = true;
}

pub struct IncrementalRenderContext<'a> {
    pub module: &'a Module,
    pub prev: &'a [Page],
    pub next: &'a [Page],
}

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Render a document difference into the svg_body.
    pub fn render_diff(&mut self, ctx: &IncrementalRenderContext<'_>, svg_body: &mut Vec<SvgText>) {
        let mut acc_height = 0u32;
        let mut render_task = self.get_render_context(ctx.module);

        let reusable: HashSet<Fingerprint, RandomState> =
            HashSet::from_iter(ctx.prev.iter().map(|e| e.content));
        let mut unused_prev: std::collections::BTreeMap<usize, Fingerprint> = ctx
            .prev
            .iter()
            .map(|e| e.content)
            .enumerate()
            .collect::<_>();

        for Page { content: entry, .. } in ctx.next.iter() {
            // todo: reuse remove unused patter, they are also used in render_diff
            if reusable.contains(entry) {
                let remove_key = unused_prev.iter().find(|(_, v)| *v == entry);
                if remove_key.is_none() {
                    continue;
                }
                unused_prev.remove(&remove_key.unwrap().0.clone());
            }
        }

        println!("reusable: {:?}", reusable);
        println!("unused_prev: {:?}", unused_prev);

        for (
            idx,
            Page {
                content: entry,
                size,
            },
        ) in ctx.next.iter().enumerate()
        {
            let size = Self::page_size(*size);
            if reusable.contains(entry) {
                println!("reuse page: {} {:?}", idx, entry);
                svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                    attributes: vec![
                        ("class", "typst-page".into()),
                        ("transform", format!("translate(0, {})", acc_height)),
                        ("data-tid", entry.as_svg_id("p")),
                        ("data-reuse-from", entry.as_svg_id("p")),
                        ("data-page-width", size.x.to_string()),
                        ("data-page-height", size.y.to_string()),
                    ],
                    content: vec![],
                })));

                acc_height += size.y;
                continue;
            }

            let mut attributes = vec![
                ("class", "typst-page".into()),
                ("transform", format!("translate(0, {})", acc_height)),
                ("data-tid", entry.as_svg_id("p")),
                ("data-page-width", size.x.to_string()),
                ("data-page-height", size.y.to_string()),
            ];

            // todo: evaluate simlarity
            let item = if let Some(prev_entry) = unused_prev.pop_first().map(|(_, v)| v) {
                println!("diff page: {} {:?} {:?}", idx, entry, prev_entry);
                attributes.push(("data-reuse-from", prev_entry.as_svg_id("p")));

                render_task.render_diff_item(entry, &prev_entry)
            } else {
                // todo: find a box
                println!("rebuild page: {} {:?}", idx, entry);
                render_task.render_flat_item(entry)
            };

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes,
                content: vec![SvgText::Content(item)],
            })));
            acc_height += size.y;
        }
    }
}

pub type IncrSvgDocServer = IncrDocServer;

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrSvgDocClient {
    /// underlying communication client model
    pub kern: IncrDocClient,

    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    pub doc_view: Option<Vec<Page>>,
    /// Glyphs that has already committed to the DOM.
    /// Assmuing glyph_window = N, then `self.doc.module.glyphs[..N]` are
    /// committed.
    pub glyph_window: usize,

    /// Don't use this
    /// it is public to make Default happy
    pub mb: ModuleBuilder,
}

impl IncrSvgDocClient {
    pub fn new(doc: MultiSvgDocument) -> Self {
        Self {
            kern: IncrDocClient {
                doc,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Kern of the client without leaking abstraction.
    pub fn kern(&self) -> IncrDocClientKern<'_> {
        IncrDocClientKern::new(&self.kern)
    }

    /// Merge the delta from server.
    pub fn merge_delta(&mut self, delta: FlatModule) {
        self.kern.merge_delta(delta);

        // checkout the current layout
        let layouts = &self.kern.doc.layouts;
        if !layouts.is_empty() {
            self.kern.set_layout(layouts.unwrap_single());
        }
    }

    fn module_mut(&mut self) -> &mut Module {
        &mut self.kern.doc.module
    }

    /// Render the document in the given window.
    pub fn render_in_window(&mut self, rect: Rect) -> String {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;

        // prepare an empty page for the pages that are not rendered
        // todo: better solution?
        let empty_page = self.mb.build(SvgItem::Group(Default::default()));
        self.kern
            .doc
            .module
            .items
            .extend(self.mb.items.iter().map(|(f, (_, v))| (*f, v.clone())));

        // get previous doc_view
        // it is exact state of the current DOM.
        let prev_doc_view = self.doc_view.take().unwrap_or_default();

        // render next doc_view
        // for pages that is not in the view, we use empty_page
        // otherwise, we keep document layout
        let mut page_off: f32 = 0.;
        let mut next_doc_view = vec![];
        if let Some(t) = &self.kern.layout {
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            for page in pages.iter() {
                page_off += page.size.y.0;
                if page_off < rect.lo.y.0 || page_off - page.size.y.0 > rect.hi.y.0 {
                    next_doc_view.push(Page {
                        content: empty_page,
                        size: page.size,
                    });
                    continue;
                }

                next_doc_view.push(page.clone());
            }
        }
        // todo: fix this

        let mut t = SvgTask::<IncrementalExportFeature>::default();

        // start to render document difference
        let mut svg = Vec::<SvgText>::new();
        svg.push(SvgText::Plain(IncrExporter::header(&next_doc_view)));

        // render the document
        let mut svg_body = vec![];
        t.render_diff(
            &IncrementalRenderContext {
                module: self.module_mut(),
                prev: &prev_doc_view,
                next: &next_doc_view,
            },
            &mut svg_body,
        );

        // render the glyphs
        svg.push(r#"<defs class="glyph">"#.into());
        let glyphs = self.kern.doc.module.glyphs.iter();
        // skip the glyphs that are already rendered
        let new_glyphs = glyphs.skip(self.glyph_window);
        let glyph_defs = t.render_glyphs(new_glyphs.enumerate().map(|(x, (_, y))| (x, y)), true);

        svg.extend(glyph_defs);
        svg.push("</defs>".into());

        // attach the clip paths, and style defs

        svg.push(r#"<defs class="clip-path">"#.into());
        IncrExporter::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".into());

        IncrExporter::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut svg_body);

        svg.push("</svg>".into());

        let mut string_io = String::new();
        string_io.reserve(svg.iter().map(SvgText::estimated_len).sum());
        for s in svg {
            s.write_string_io(&mut string_io);
        }

        // update the state
        self.doc_view = Some(next_doc_view);
        self.glyph_window = self.module_mut().glyphs.len();

        // return the svg
        string_io
    }
}
