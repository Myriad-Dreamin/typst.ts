use std::sync::Arc;

use typst::doc::Document;
use typst_ts_core::vector::{
    flat_ir::{
        build_flat_glyphs, serialize_module_v2, IncrModuleBuilder, Module, ModuleMetadata, Pages,
        SerializedModule, SourceMappingNode, SvgDocument,
    },
    ir::Scalar,
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
pub struct IncrementalSvgV2Exporter {
    prev: Option<SvgDocument>,
    module_builder: IncrModuleBuilder,
    page_source_mapping: Vec<SourceMappingNode>,

    should_attach_debug_info: bool,
}

impl IncrementalSvgV2Exporter {
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.module_builder.should_attach_debug_info = should_attach_debug_info;
        self.should_attach_debug_info = should_attach_debug_info;
    }

    pub fn pack_delta(&mut self, output: Arc<Document>) -> Vec<u8> {
        self.module_builder.reset();
        self.page_source_mapping.clear();

        let instant: std::time::Instant = std::time::Instant::now();

        self.module_builder.increment_lifetime();

        // it is important to call gc before building pages
        let gc_items = self.module_builder.gc(120);

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
        let (module, glyph_mapping) = builder.finalize_delta();

        let glyphs = build_flat_glyphs(glyph_mapping);

        let delta = serialize_module_v2(&SerializedModule {
            metadata: vec![
                ModuleMetadata::SourceMappingData(module.source_mapping),
                ModuleMetadata::PageSourceMapping(vec![self.page_source_mapping.clone()]),
                ModuleMetadata::GarbageCollection(gc_items),
            ],
            glyphs,
            item_pack: module.item_pack,
            layouts: vec![(Scalar(0.), pages.clone())],
        });

        println!("svg render time (incremental bin): {:?}", instant.elapsed());

        [b"diff-v1,", delta.as_slice()].concat()
    }

    pub fn pack_current(&mut self) -> Option<Vec<u8>> {
        let doc = self.prev.as_ref()?;
        let glyphs = build_flat_glyphs(self.module_builder.glyphs.clone());

        let delta = serialize_module_v2(&SerializedModule {
            metadata: vec![
                ModuleMetadata::SourceMappingData(self.module_builder.source_mapping.clone()),
                ModuleMetadata::PageSourceMapping(vec![self.page_source_mapping.clone()]),
            ],
            glyphs,
            item_pack: doc.module.item_pack.clone(),
            layouts: vec![(Scalar(0.), doc.pages.clone())],
        });
        Some([b"diff-v1,", delta.as_slice()].concat())
    }

    pub fn render_in_window(module: &Module, prev: Option<Pages>, next: &Pages) -> String {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;

        let mut svg = Vec::<SvgText>::new();
        svg.push(SvgText::Plain(IncrExporter::header(next)));
        let mut svg_body = vec![];

        // render the document
        let mut t = SvgTask::<IncrementalExportFeature>::default();

        let prev = prev.unwrap_or_default();
        let render_context = IncrementalRenderContext {
            module,
            prev: &prev,
            next,
        };
        t.render_diff(&render_context, &mut svg_body);

        // todo: render glyphs
        // svg.push(r#"<defs class="glyph">"#.into());
        // svg.extend(new_glyphs);
        // svg.push("</defs>".into());

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

        string_io
    }
}
