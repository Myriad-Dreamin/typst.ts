use core::fmt;
use std::{
    collections::{hash_map::RandomState, HashMap, HashSet},
    sync::Arc,
};

use typst::doc::Document;

use crate::{
    flat_incr_vector::FlatIncrRenderVm,
    flat_vector::{FlatRenderVm, ItemPack, SourceMappingNode, SvgDocument},
    ir::AbsoluteRef,
    vector::{
        codegen::{SvgText, SvgTextNode},
        lowering::LowerBuilder,
    },
    ExportFeature, Module, ModuleBuilder, SvgExporter, SvgTask,
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

        let reusable: HashSet<AbsoluteRef, RandomState> =
            HashSet::from_iter(ctx.prev.pages.iter().map(|e| e.0.clone()));
        let mut unused_prev: std::collections::BTreeMap<usize, AbsoluteRef> = ctx
            .prev
            .pages
            .iter()
            .map(|e| e.0.clone())
            .enumerate()
            .collect::<_>();

        for (entry, _) in ctx.next.pages.iter() {
            if reusable.contains(entry) {
                let remove_key = unused_prev.iter().find(|(_, v)| *v == entry).unwrap().0;
                unused_prev.remove(&remove_key.clone());
            }
        }

        println!("reusable: {:?}", reusable);
        println!("unused_prev: {:?}", unused_prev);

        for (idx, (entry, size)) in ctx.next.pages.iter().enumerate() {
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

            let item = if let Some(prev_entry) = unused_prev.pop_first().map(|(_, v)| v) {
                println!("diff page: {} {:?} {:?}", idx, entry, prev_entry);
                render_task.render_diff_item(entry, &prev_entry)
            } else {
                println!("rebuild page: {} {:?}", idx, entry);
                render_task.render_flat_item(entry)
            };

            let mut attributes = vec![
                ("class", "typst-page".into()),
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

#[derive(Default)]
pub struct IncrementalSvgExporter {
    prev: Option<SvgDocument>,
    module_builder: ModuleBuilder,
    page_source_mapping: Vec<SourceMappingNode>,

    should_attach_debug_info: bool,
}

struct SrcMappingRepr<'a> {
    mapping: &'a [SourceMappingNode],
}

impl fmt::Display for SrcMappingRepr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut st = false;
        for e in self.mapping {
            if st {
                write!(f, "|")?;
            } else {
                st = true;
            }
            match e {
                SourceMappingNode::Page(p) => write!(f, "p,{:x}", p)?,
                SourceMappingNode::Text(t) => write!(f, "t,{:x}", t)?,
                SourceMappingNode::Image(i) => write!(f, "i,{:x}", i)?,
                SourceMappingNode::Shape(s) => write!(f, "s,{:x}", s)?,
                SourceMappingNode::Group(refs) => {
                    f.write_str("g")?;
                    for r in refs.iter() {
                        write!(f, ",{:x}", r)?;
                    }
                }
            }
        }

        Ok(())
    }
}

fn repr_src_mapping(mapping: &[SourceMappingNode]) -> SrcMappingRepr<'_> {
    SrcMappingRepr { mapping }
}

impl IncrementalSvgExporter {
    pub fn set_should_attach_debug_info(&mut self, should_attach_debug_info: bool) {
        self.module_builder.should_attach_debug_info = should_attach_debug_info;
        self.should_attach_debug_info = should_attach_debug_info;
    }

    fn render_source_mapping(&self) -> String {
        if !self.should_attach_debug_info {
            return String::new();
        }
        let entire = &self.module_builder.source_mapping;
        let t = &self.page_source_mapping;
        format!(
            r#"<div class="typst-source-mapping" data-pages="{}" data-source-mapping="{}">"#,
            repr_src_mapping(t),
            repr_src_mapping(entire)
        )
    }

    fn render_svg_incremental(
        &mut self,
        prev: SvgDocument,
        output: Arc<Document>,
    ) -> (SvgDocument, String) {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;
        self.page_source_mapping.clear();

        let instant: std::time::Instant = std::time::Instant::now();

        // render the document
        let mut t = SvgTask::<IncrementalExportFeature>::default();

        let (next, used_glyphs) = {
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
            let (module, glyph_mapping) = builder.finalize_ref();

            (SvgDocument { pages, module }, glyph_mapping)
        };

        let mut svg = Vec::<SvgText>::new();
        svg.push(SvgText::Plain(IncrExporter::header(&next.pages)));
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
        svg.push(r#"<defs class="glyph">"#.into());
        svg.extend(new_glyphs);
        svg.push("</defs>".into());

        svg.push(r#"<defs class="clip-path">"#.into());
        IncrExporter::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".into());

        IncrExporter::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut svg_body);

        svg.push("</svg>".into());

        println!("svg render time (incremental): {:?}", instant.elapsed());

        let mut string_io = String::new();
        string_io.reserve(svg.iter().map(SvgText::estimated_len).sum());
        for s in svg {
            s.write_string_io(&mut string_io);
        }
        (svg_doc, string_io)
    }

    pub fn render(&mut self, output: Arc<Document>) -> String {
        self.module_builder.reset();
        let (next, packet) = match self.prev.take() {
            Some(prev) => {
                let (next, svg) = self.render_svg_incremental(prev, output);
                (
                    next,
                    ["diff-v0,", &svg, &self.render_source_mapping()].concat(),
                )
            }
            None => {
                let (next, svg) = self.render_svg_incremental(
                    SvgDocument {
                        module: Module {
                            glyphs: vec![],
                            item_pack: ItemPack::default(),
                            source_mapping: Default::default(),
                        },
                        pages: vec![],
                    },
                    output,
                );

                (next, ["new,", &svg, &self.render_source_mapping()].concat())
            }
        };

        self.prev = Some(next);
        packet
    }

    pub fn render_current(&mut self) -> Option<String> {
        type IncrExporter = SvgExporter<IncrementalExportFeature>;
        let doc = self.prev.as_ref()?;

        let svg = IncrExporter::render_flat_svg(&doc.module, &doc.pages);
        Some(["new,", &svg, &self.render_source_mapping()].concat())
    }
}
