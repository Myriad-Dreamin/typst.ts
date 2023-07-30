use std::sync::Arc;

use typst::doc::Document;
use typst_ts_core::vector::{
    flat_ir::{
        flatten_glyphs, FlatModule, IncrModuleBuilder, ItemPack, ModuleBuilder, ModuleMetadata,
        MultiSvgDocument, Pages, SourceMappingNode, SvgDocument,
    },
    ir::{Rect, Scalar, SvgItem},
    LowerBuilder,
};

use crate::{backend::SvgText, ExportFeature, IncrementalRenderContext, SvgExporter, SvgTask};

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

#[derive(Default)]
pub struct IncrSvgDocServer {
    /// Whether to attach debug info to the output.
    should_attach_debug_info: bool,

    /// Expected exact state of the current Compiler.
    /// Initially it is None meaning no completed compilation.
    doc_view: Option<SvgDocument>,

    /// Maintaining document build status
    module_builder: IncrModuleBuilder,

    /// Optional page source mapping references.
    page_source_mapping: Vec<SourceMappingNode>,
}

impl IncrSvgDocServer {
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.module_builder.should_attach_debug_info = should_attach_debug_info;
        self.should_attach_debug_info = should_attach_debug_info;
    }

    /// Pack the delta into a binary blob.
    pub fn pack_delta(&mut self, output: Arc<Document>) -> Vec<u8> {
        self.module_builder.reset();
        self.page_source_mapping.clear();

        let instant: std::time::Instant = std::time::Instant::now();

        self.module_builder.increment_lifetime();

        // it is important to call gc before building pages
        let gc_items = self.module_builder.gc(5 * 2);

        let mut lower_builder = LowerBuilder::new(&output);
        let builder = &mut self.module_builder;
        let pages = output
            .pages
            .iter()
            .map(|p| {
                let abs_ref = builder.build(lower_builder.lower(p));
                if self.should_attach_debug_info {
                    self.page_source_mapping.push(SourceMappingNode::Page(
                        (builder.source_mapping.len() - 1) as u64,
                    ));
                }
                (abs_ref, p.size().into())
            })
            .collect::<Vec<_>>();
        let delta = builder.finalize_delta();

        let glyphs = flatten_glyphs(delta.glyphs.into_iter().map(|(x, y)| (y, x)));

        // max, min lifetime current, gc_items
        #[cfg(feature = "debug_gc")]
        println!(
            "gc: max: {}, min: {}, curr: {}, {}",
            self.module_builder
                .items
                .values()
                .map(|i| i.0)
                .max()
                .unwrap_or(0xffffffff),
            self.module_builder
                .items
                .values()
                .map(|i| i.0)
                .min()
                .unwrap_or(0),
            self.module_builder.lifetime,
            gc_items.len()
        );
        let delta = FlatModule {
            metadata: vec![
                ModuleMetadata::SourceMappingData(delta.source_mapping),
                ModuleMetadata::PageSourceMapping(vec![self.page_source_mapping.clone()]),
                ModuleMetadata::GarbageCollection(gc_items),
            ],
            glyphs,
            item_pack: ItemPack(delta.items.clone().into_iter().collect()),
            layouts: vec![(Scalar(0.), pages.clone())],
        }
        .to_bytes();

        println!("svg render time (incremental bin): {:?}", instant.elapsed());

        [b"diff-v1,", delta.as_slice()].concat()
    }

    /// Pack the current entirely into a binary blob.
    pub fn pack_current(&mut self) -> Option<Vec<u8>> {
        let doc = self.doc_view.as_ref()?;
        let glyphs = flatten_glyphs(self.module_builder.glyphs.clone());

        let delta = FlatModule {
            metadata: vec![
                ModuleMetadata::SourceMappingData(self.module_builder.source_mapping.clone()),
                ModuleMetadata::PageSourceMapping(vec![self.page_source_mapping.clone()]),
            ],
            glyphs,
            item_pack: ItemPack(doc.module.items.clone().into_iter().collect()),
            layouts: vec![(Scalar(0.), doc.pages.clone())],
        }
        .to_bytes();
        Some([b"diff-v1,", delta.as_slice()].concat())
    }
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrSvgDocClient {
    /// Full information of the current document from server.
    pub doc: MultiSvgDocument,

    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    pub doc_view: Option<Pages>,
    /// Glyphs that has already committed to the DOM.
    /// Assmuing glyph_window = N, then `self.doc.module.glyphs[..N]` are
    /// committed.
    pub glyph_window: usize,

    /// Optional source mapping data.
    pub source_mapping_data: Vec<SourceMappingNode>,
    /// Optional page source mapping references.
    pub page_source_mappping: Vec</* layout pages */ Vec</* per page */ SourceMappingNode>>,

    mb: ModuleBuilder,
}

impl IncrSvgDocClient {
    /// Merge the delta from server.
    pub fn merge_delta(&mut self, delta: FlatModule) {
        self.doc.merge_delta(&delta);
        for metadata in delta.metadata {
            match metadata {
                ModuleMetadata::SourceMappingData(data) => {
                    self.source_mapping_data = data;
                }
                ModuleMetadata::PageSourceMapping(data) => {
                    self.page_source_mappping = data;
                }
                _ => {}
            }
        }
    }

    /// Render the document in the given window.
    pub fn render_in_window(&mut self, rect: Rect) -> String {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;

        // prepare an empty page for the pages that are not rendered
        // todo: better solution?
        let empty_page = self.mb.build(SvgItem::Group(Default::default()));
        self.doc
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
        if !self.doc.layouts.is_empty() {
            for (page, layout) in self.doc.layouts[0].1.iter() {
                page_off += layout.y.0;
                if page_off < rect.lo.y.0 || page_off - layout.y.0 > rect.hi.y.0 {
                    next_doc_view.push((empty_page, *layout));
                    continue;
                }

                next_doc_view.push((*page, *layout));
            }
        }

        let mut t = SvgTask::<IncrementalExportFeature>::default();

        // start to render document difference
        let mut svg = Vec::<SvgText>::new();
        svg.push(SvgText::Plain(IncrExporter::header(&next_doc_view)));

        // render the document
        let mut svg_body = vec![];
        t.render_diff(
            &IncrementalRenderContext {
                module: &self.doc.module,
                prev: &prev_doc_view,
                next: &next_doc_view,
            },
            &mut svg_body,
        );

        // render the glyphs
        svg.push(r#"<defs class="glyph">"#.into());
        let glyphs = self.doc.module.glyphs.iter();
        // skip the glyphs that are already rendered
        let new_glyphs = glyphs.skip(self.glyph_window);
        let glyph_defs = t.render_glyphs(new_glyphs.map(|g| (&g.0, &g.1)), true);

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
        self.glyph_window = self.doc.module.glyphs.len();

        // return the svg
        string_io
    }
}
