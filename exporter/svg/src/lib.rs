//! Rendering into svg text or module.

pub(crate) use tiny_skia as sk;

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use typst::diag::SourceResult;
use typst::doc::Document;
use typst::World;

use typst_ts_core::font::{FontGlyphProvider, GlyphProvider};
use typst_ts_core::Exporter;

use geom::{Axes, Size};
use ir::{
    Abs, AbsoulteRef, GlyphMapping, ImmutStr, Module, ModuleBuilder, Pages, StyleNs, SvgDocument,
};
use lowering::{GlyphLowerBuilder, LowerBuilder};
use render::SvgRenderTask;
use vm::RenderVm;

pub(crate) mod escape;
pub mod geom;
pub(crate) mod ir;
pub(crate) mod lowering;
pub(crate) mod render;
pub(crate) mod svg;
pub(crate) mod utils;
pub(crate) mod vm;
pub use ir::LayoutElem;
pub use ir::MultiSvgDocument;
pub use ir::SerializedModule;

pub trait ExportFeature {
    const ENABLE_TRACING: bool;
}

pub struct DefaultExportFeature;
pub type DefaultSvgTask = SvgTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
}

pub struct IncrementalRenderContext {
    prev: SvgDocument,
    next: SvgDocument,
}

type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
type ClipPathMap = HashMap<ImmutStr, u32>;

pub struct SvgTask<Feat: ExportFeature = DefaultExportFeature> {
    glyph_provider: GlyphProvider,
    style_defs: StyleDefMap,
    clip_paths: ClipPathMap,

    // errors: Vec<Error>,
    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: ExportFeature> SvgTask<Feat> {
    pub fn new() -> Self {
        let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        Self {
            glyph_provider,
            style_defs: HashMap::default(),
            clip_paths: HashMap::default(),

            _feat_phantom: Default::default(),
        }
    }

    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    pub fn page_size(sz: Size) -> Axes<u32> {
        let (width_px, height_px) = {
            let width_px = (sz.x.0.ceil()).round().max(1.0) as u32;
            let height_px = (sz.y.0.ceil()).round().max(1.0) as u32;

            (width_px, height_px)
        };

        Axes::new(width_px, height_px)
    }

    pub fn fork_render_task<'m, 't>(
        &'t mut self,
        module: &'m ir::Module,
    ) -> SvgRenderTask<'m, 't, DefaultExportFeature> {
        SvgRenderTask::<DefaultExportFeature> {
            glyph_provider: self.glyph_provider.clone(),

            module,
            page_off: 0,

            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_render_text_element: true,

            _feat_phantom: Default::default(),
        }
    }

    /// Render a document into the svg_body.
    fn render_glyphs(&mut self, module: &Module, svg_body: &mut Vec<String>) {
        let mut render_task = self.fork_render_task(module);

        for (abs_ref, item) in module.glyphs.iter() {
            svg_body.push(render_task.render_glyph(abs_ref, item).unwrap_or_default())
        }
    }

    /// Render a document into the svg_body.
    pub fn render(&mut self, module: &Module, pages: &Pages, svg_body: &mut Vec<String>) {
        let mut render_task = self.fork_render_task(module);

        let mut acc_height = 0u32;
        for (idx, page) in pages.iter().enumerate() {
            render_task.page_off = idx;

            let entry = &page.0;
            let size = Self::page_size(page.1);
            let item = render_task.render_item(entry);
            let item = format!(
                r#"<g transform="translate(0, {})" data-tid="{}" data-page-width="{}" data-page-height="{}">{}</g>"#,
                acc_height,
                entry.as_svg_id("p"),
                size.x,
                size.y,
                item
            );

            svg_body.push(item);
            acc_height += size.y;
        }
    }

    /// Render a document difference into the svg_body.
    pub fn render_diff(&mut self, ctx: &IncrementalRenderContext, svg_body: &mut Vec<String>) {
        let mut acc_height = 0u32;
        let mut render_task = self.fork_render_task(&ctx.next.module);

        let reusable: HashSet<AbsoulteRef, RandomState> =
            HashSet::from_iter(ctx.prev.pages.iter().map(|e| e.0.clone()));

        for (idx, (entry, size)) in ctx.next.pages.iter().enumerate() {
            render_task.page_off = idx;

            let size = Self::page_size(*size);
            if reusable.contains(entry) {
                let item: String = format!(
                    r#"<g transform="translate(0, {})" data-tid="{}" data-reuse-from="{}" data-page-width="{}" data-page-height="{}"></g>"#,
                    acc_height,
                    entry.as_svg_id("p"),
                    entry.as_svg_id("p"),
                    size.x,
                    size.y,
                );

                svg_body.push(item);
                acc_height += size.y;
                continue;
            }

            let item = render_task.render_item(entry);

            // todo: evaluate simlarity
            let reuse_info = match ctx.prev.pages.get(idx) {
                Some((abs_ref, ..)) => {
                    format!(r#" data-reuse-from="{}""#, abs_ref.as_svg_id("p"))
                }
                None => String::new(),
            };

            let item: String = format!(
                r#"<g transform="translate(0, {})" data-tid="{}"{} data-page-width="{}" data-page-height="{}">{}</g>"#,
                acc_height,
                entry.as_svg_id("p"),
                reuse_info,
                size.x,
                size.y,
                item
            );

            svg_body.push(item);
            acc_height += size.y;
        }
    }
}

impl<Feat: ExportFeature> Default for SvgTask<Feat> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct SvgExporter {}

impl SvgExporter {
    fn header(output: &Pages) -> String {
        // calculate the width and height of the svg
        let w = output
            .iter()
            .map(|p| p.1.x.0.ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output.iter().map(|p| p.1.y.0.ceil()).sum::<f32>();

        format!(
            r#"<svg class="typst-doc" viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        )
    }

    fn svg_doc(output: &Document) -> (SvgDocument, GlyphMapping) {
        let mut lower_builder = LowerBuilder::new(output);
        let mut builder = ModuleBuilder::default();
        let pages = output
            .pages
            .iter()
            .map(|p| {
                let abs_ref = builder.build(lower_builder.lower(p));
                (abs_ref, p.size().into())
            })
            .collect::<Vec<_>>();
        let (module, glyph_mapping) = builder.finalize();

        (SvgDocument { pages, module }, glyph_mapping)
    }

    fn style_defs(style_defs: StyleDefMap, svg: &mut Vec<String>) {
        // style defs
        svg.push(r#"<style type="text/css">"#.to_owned());
        let mut g = style_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(g.into_iter().map(|v| v.1));
        svg.push("</style>".to_owned());
    }

    fn clip_paths(clip_paths: ClipPathMap, svg: &mut Vec<String>) {
        let mut g = clip_paths.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in g {
            svg.push(format!(
                r##"<clipPath id="c{:x}"><path d="{}"/></clipPath>"##,
                id, clip_path
            ));
        }
    }

    pub fn render(module: &Module, pages: &Pages) -> String {
        let mut svg = vec![Self::header(pages)];
        let mut svg_body = vec![];

        let mut t = SvgTask::<DefaultExportFeature>::new();
        t.render(module, pages, &mut svg_body);

        // base style
        svg.push(r#"<style type="text/css">"#.to_owned());
        svg.push(include_str!("./typst.svg.css").to_owned());
        svg.push("</style>".to_owned());

        // attach the glyph defs, clip paths, and style defs
        svg.push("<defs>".to_owned());
        svg.push("<g>".to_owned());
        t.render_glyphs(module, &mut svg);
        svg.push("</g>".to_owned());
        Self::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".to_owned());
        Self::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript">"#.to_owned());
        // svg.push(r#"<![CDATA["#.to_owned());
        svg.push(include_str!("./typst.svg.js").to_owned());
        // svg.push(r#"]]>"#.to_owned());
        svg.push("</script>".to_owned());

        // close the svg
        svg.push("</svg>".to_owned());

        svg.join("")
    }

    fn render_svg(output: Arc<Document>) -> (SvgDocument, String) {
        let instant = std::time::Instant::now();
        // render the document
        let (doc, _used_glyphs) = Self::svg_doc(&output);

        let svg = Self::render(&doc.module, &doc.pages);
        println!("svg render time: {:?}", instant.elapsed());
        (doc, svg)
    }

    fn render_svg_incremental(prev: SvgDocument, output: Arc<Document>) -> (SvgDocument, String) {
        let instant = std::time::Instant::now();

        // render the document
        let mut t = SvgTask::<DefaultExportFeature>::new();

        let (next, used_glyphs) = Self::svg_doc(&output);

        let mut svg = vec![Self::header(&next.pages)];
        let mut svg_body = vec![];

        let render_context = IncrementalRenderContext { prev, next };
        t.render_diff(&render_context, &mut svg_body);
        let svg_doc = render_context.next;

        // base style
        svg.push(r#"<style type="text/css" data-reuse="1">"#.to_owned());
        svg.push("</style>".to_owned());

        // attach the glyph defs, clip paths, and style defs
        svg.push("<defs>".to_owned());
        let _ = used_glyphs;

        svg.push("</defs>".to_owned());

        // incremental style
        svg.push(r#"<style type="text/css" data-reuse="1">"#.to_owned());
        svg.push("</style>".to_owned());

        // body
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript" data-reuse="1">"#.to_owned());
        svg.push("</script>".to_owned());

        svg.push("</svg>".to_owned());

        println!("svg render time (incremental): {:?}", instant.elapsed());
        (svg_doc, svg.join(""))
    }
}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        let svg = Self::render_svg(output.clone()).1;
        // html wrap
        Ok(format!(
            r#"<html><head><meta charset="utf-8" /><title>{}</title></head><body>{}</body></html>"#,
            output
                .title
                .clone()
                .unwrap_or_else(|| "Typst Document".into()),
            svg
        ))
    }
}

#[derive(Default)]
pub struct IncrementalSvgExporter {
    prev: Option<SvgDocument>,
}

impl IncrementalSvgExporter {
    pub fn render(&mut self, output: Arc<Document>) -> String {
        let (next, packet) = match self.prev.take() {
            Some(prev) => {
                let (next, svg) = SvgExporter::render_svg_incremental(prev, output);
                (next, ["diff-v0,", &svg].concat())
            }
            None => {
                let (next, svg) = SvgExporter::render_svg(output);
                (next, ["new,", &svg].concat())
            }
        };

        self.prev = Some(next);
        packet
    }
}

#[derive(Default)]
pub struct DynamicLayoutSvgExporter {
    builder: ModuleBuilder,
    layouts: Vec<(Abs, Vec<(AbsoulteRef, Size)>)>,
}

impl DynamicLayoutSvgExporter {
    pub fn render(&mut self, layout_width: typst::geom::Abs, output: Arc<Document>) {
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
        let item_cnt = v.0.item_pack.0.len();
        let glyph_cnt = v.1.len();
        let module_data = serialize_module(v.0);
        format!(
            "module size: {} bytes, items count: {}, glyph count: {}",
            module_data.len(),
            item_cnt,
            glyph_cnt
        )
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<Document, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<Vec<u8>> {
        let mut t = LowerBuilder::new(&output);

        let mut builder = ModuleBuilder::default();

        for page in output.pages.iter() {
            let item = t.lower(page);
            let _entry_id = builder.build(item);
        }

        let (repr, ..) = builder.finalize();

        Ok(serialize_module(repr))
    }
}

fn serialize_module(repr: Module) -> Vec<u8> {
    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(&repr.item_pack).unwrap();
    let item_pack = serializer.into_serializer().into_inner();

    item_pack.into_vec()
}

pub fn serialize_multi_doc_standalone(
    doc: MultiSvgDocument,
    glyph_mapping: GlyphMapping,
) -> Vec<u8> {
    let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
    let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider);

    let glyphs = glyph_mapping
        .into_iter()
        .filter_map(|(glyph, glyph_id)| {
            let glyph = glyph_lower_builder.lower_glyph(&glyph);
            glyph.map(|t| {
                let t = match t {
                    ir::GlyphItem::Image(i) => ir::FlatGlyphItem::Image(i),
                    ir::GlyphItem::Outline(p) => ir::FlatGlyphItem::Outline(p),
                    _ => unreachable!(),
                };

                (glyph_id, t)
            })
        })
        .collect::<Vec<_>>();

    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::{serializers::AllocSerializer, Serializer};

    let mut serializer = AllocSerializer::<0>::default();
    serializer
        .serialize_value(&SerializedModule {
            item_pack: doc.module.item_pack,
            glyphs,
            layouts: doc.layouts,
        })
        .unwrap();
    let item_pack = serializer.into_serializer().into_inner();

    item_pack.into_vec()
}
