use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    sync::Arc,
};

use typst::doc::Document;

use crate::{
    flat_vector::{FlatRenderVm, SvgDocument},
    ir::AbsoulteRef,
    vector::codegen::{SvgText, SvgTextNode},
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
}

pub struct IncrementalRenderContext {
    prev: SvgDocument,
    next: SvgDocument,
}

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Render a document difference into the svg_body.
    pub fn render_diff(&mut self, ctx: &IncrementalRenderContext, svg_body: &mut Vec<SvgText>) {
        let mut acc_height = 0u32;
        let mut render_task = self.fork_page_render_task(&ctx.next.module);

        let reusable: HashSet<AbsoulteRef, RandomState> =
            HashSet::from_iter(ctx.prev.pages.iter().map(|e| e.0.clone()));

        for (idx, (entry, size)) in ctx.next.pages.iter().enumerate() {
            let size = Self::page_size(*size);
            if reusable.contains(entry) {
                svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                    attributes: vec![
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

            let item = render_task.render_flat_item(entry);

            let mut attributes = vec![
                ("transform", format!("translate(0, {})", acc_height)),
                ("data-tid", entry.as_svg_id("p")),
                ("data-page-width", size.x.to_string()),
                ("data-page-height", size.y.to_string()),
            ];

            // todo: evaluate simlarity
            if let Some((abs_ref, ..)) = ctx.prev.pages.get(idx) {
                attributes.push(("data-reuse-from", abs_ref.as_svg_id("p")));
            }

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes,
                content: vec![SvgText::Content(item)],
            })));
            acc_height += size.y;
        }
    }
}

impl<Feat: ExportFeature> SvgExporter<Feat> {
    fn render_svg_incremental(prev: SvgDocument, output: Arc<Document>) -> (SvgDocument, String) {
        let instant = std::time::Instant::now();

        // render the document
        let mut t = SvgTask::<IncrementalExportFeature>::default();

        let (next, used_glyphs) = Self::svg_doc(&output);

        let mut svg = Vec::<SvgText>::new();
        svg.push(SvgText::Plain(Self::header(&next.pages)));
        let mut svg_body = vec![];

        let new_glyphs = {
            let prev_glyphs = prev
                .module
                .glyphs
                .iter()
                .cloned()
                .map(|(x, y)| (y, x))
                .collect::<HashMap<_, _>>();
            let new_glyphs = used_glyphs
                .iter()
                .filter(|(g, _)| !prev_glyphs.contains_key(g))
                .map(|(x, y)| (y, x));
            t.render_glyphs(new_glyphs, true)
        };

        let render_context = IncrementalRenderContext { prev, next };
        t.render_diff(&render_context, &mut svg_body);
        let svg_doc = render_context.next;

        // attach the glyph defs, clip paths, and style defs
        svg.push(r#"<defs id="glyph">"#.into());
        svg.extend(new_glyphs);
        svg.push("</defs>".into());

        svg.push(r#"<defs id="clip-path">"#.into());
        Self::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".into());

        Self::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript" data-reuse="1">"#.into());
        svg.push("</script>".into());

        svg.push("</svg>".into());

        println!("svg render time (incremental): {:?}", instant.elapsed());

        let mut string_io = String::new();
        string_io.reserve(svg.iter().map(SvgText::estimated_len).sum());
        for s in svg {
            s.write_string_io(&mut string_io);
        }
        (svg_doc, string_io)
    }
}

#[derive(Default)]
pub struct IncrementalSvgExporter {
    prev: Option<SvgDocument>,
}

impl IncrementalSvgExporter {
    pub fn render(&mut self, output: Arc<Document>) -> String {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;

        let (next, packet) = match self.prev.take() {
            Some(prev) => {
                let (next, svg) = IncrExporter::render_svg_incremental(prev, output);
                (next, ["diff-v0,", &svg].concat())
            }
            None => {
                let (next, svg) = IncrExporter::render_flat_doc_and_svg(output);
                (next, ["new,", &svg].concat())
            }
        };

        self.prev = Some(next);
        packet
    }
}
