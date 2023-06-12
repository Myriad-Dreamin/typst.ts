//! Rendering into svg text or module.

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use ir::{AbsoulteRef, ModuleBuilder, RelativeRef, StyleNs};
use render::SvgRenderTask;

use typst::diag::SourceResult;
use typst::doc::{Document, Frame};
use typst::geom::Axes;
use typst::World;
use typst_ts_core::annotation::link::AnnotationProcessor;
use typst_ts_core::error::prelude::*;
use typst_ts_core::font::{FontGlyphProvider, GlyphProvider};
use typst_ts_core::Exporter;

pub(crate) mod ir;
pub(crate) mod lowering;
pub(crate) mod render;
pub(crate) mod svg;
pub(crate) mod utils;

pub trait RenderFeature {
    const ENABLE_TRACING: bool;
}

pub struct DefaultRenderFeature;

impl RenderFeature for DefaultRenderFeature {
    const ENABLE_TRACING: bool = false;
}

pub struct SvgDocument {
    pub module: ir::Module,
    pub pages: Vec<AbsoulteRef>,
}

pub type RenderContext = (Arc<Document>, SvgDocument);
pub type IncrementalRenderContext = (Arc<Document>, SvgDocument, SvgDocument);
type GlyphDefMap = HashMap<String, String>;
type ClipPathMap = HashMap<Arc<str>, u32>;

pub struct SvgTask<Feat: RenderFeature = DefaultRenderFeature> {
    glyph_provider: GlyphProvider,
    annotation_proc: AnnotationProcessor,

    style_defs: HashMap<(StyleNs, Arc<str>), String>,
    glyph_defs: GlyphDefMap,
    clip_paths: ClipPathMap,

    // errors: Vec<Error>,
    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: RenderFeature> SvgTask<Feat> {
    pub fn new(doc: &Document) -> ZResult<Self> {
        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        Ok(Self {
            glyph_provider: default_glyph_provider,
            annotation_proc: AnnotationProcessor::new(doc),
            style_defs: HashMap::default(),
            glyph_defs: HashMap::default(),
            clip_paths: HashMap::default(),

            _feat_phantom: Default::default(),
        })
    }

    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    pub fn page_size(frame: &Frame) -> Axes<u32> {
        let (width_px, height_px) = {
            let size = frame.size();
            let width_px = (size.x.to_pt().ceil() as f32).round().max(1.0) as u32;
            let height_px = (size.y.to_pt().ceil() as f32).round().max(1.0) as u32;

            (width_px, height_px)
        };

        Axes::new(width_px, height_px)
    }

    /// Render a document into the svg_body.
    pub fn render(&mut self, input: &RenderContext, svg_body: &mut Vec<String>) -> ZResult<()> {
        let mut acc_height = 0f32;
        for (idx, page) in input.0.pages.iter().enumerate() {
            let entry = &input.1.pages[idx];
            let size = Self::page_size(page);
            let item = self
                .render_frame(idx, &input.1.module, page, entry.clone())
                .unwrap();
            let item = format!(
                r#"<g transform="translate(0, {})" data-tid="{}">{}</g>"#,
                acc_height,
                entry.as_svg_id("p"),
                item
            );

            svg_body.push(item);
            acc_height += size.y as f32;
        }

        Ok(())
    }

    /// Render a document into the svg_body.
    pub fn render_diff(
        &mut self,
        (typst_doc, prev_doc, next_doc): &IncrementalRenderContext,
        svg_body: &mut Vec<String>,
    ) -> ZResult<()> {
        let mut acc_height = 0f32;

        let reusable: HashSet<RelativeRef, RandomState> =
            HashSet::from_iter(prev_doc.pages.clone().into_iter().map(|e| {
                let id = e.id;
                id.make_relative_ref(e)
            }));
        for (idx, page) in typst_doc.pages.iter().enumerate() {
            let entry = &next_doc.pages[idx];
            let relative_entry = entry.id.make_relative_ref(entry.clone());
            let size = Self::page_size(page);
            if reusable.contains(&relative_entry) {
                let item: String = format!(
                    r#"<g transform="translate(0, {})" data-tid="{}" data-reuse-from="{}"></g>"#,
                    acc_height,
                    relative_entry.as_svg_id("p"),
                    relative_entry.as_svg_id("p"),
                );

                svg_body.push(item);
                acc_height += size.y as f32;
                continue;
            }

            let item = self
                .render_frame(idx, &next_doc.module, page, entry.clone())
                .unwrap();

            // todo: evaluate simlarity
            let reuse_info = match prev_doc.pages.get(idx) {
                Some(abs_ref) => {
                    let prev_relative_entry = abs_ref.id.make_relative_ref(abs_ref.clone());
                    format!(
                        r#" data-reuse-from="{}""#,
                        prev_relative_entry.as_svg_id("p")
                    )
                }
                None => String::new(),
            };

            let item: String = format!(
                r#"<g transform="translate(0, {})" data-tid="{}"{}>{}</g>"#,
                acc_height,
                relative_entry.as_svg_id("p"),
                reuse_info,
                item
            );

            svg_body.push(item);
            acc_height += size.y as f32;
        }

        Ok(())
    }

    /// Render a frame into the a `<g/>` element.
    pub fn render_frame(
        &mut self,
        idx: usize,
        module: &ir::Module,
        frame: &Frame,
        entry: AbsoulteRef,
    ) -> ZResult<String> {
        let size = Self::page_size(frame);

        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        let mut t = SvgRenderTask::<DefaultRenderFeature> {
            glyph_provider: default_glyph_provider,

            module,

            style_defs: &mut self.style_defs,
            glyph_defs: &mut self.glyph_defs,
            clip_paths: &mut self.clip_paths,
            render_text_element: true,

            page_off: idx,
            width_px: size.x,
            height_px: size.y,
            raw_height: size.x as f32,

            font_map: HashMap::default(),

            _feat_phantom: Default::default(),
        };

        t.render_item(entry)
    }
}

#[derive(Default)]
pub struct SvgExporter {}

impl SvgExporter {
    fn header(output: &Document) -> String {
        // calculate the width and height of the svg
        let w = output
            .pages
            .iter()
            .map(|p| p.width().to_pt().ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output
            .pages
            .iter()
            .map(|p| p.height().to_pt().ceil())
            .sum::<f64>();

        format!(
            r#"<svg viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        )
    }

    fn svg_doc<Feat: RenderFeature>(
        task_context: &mut SvgTask<Feat>,
        output: &Document,
    ) -> SvgDocument {
        let mut builder = ModuleBuilder::default();
        let pages = output
            .pages
            .iter()
            .map(|p| builder.build(task_context.lower(p)))
            .collect::<Vec<_>>();
        let module = builder.finalize();

        SvgDocument { pages, module }
    }

    fn glyph_defs(glyph_defs: GlyphDefMap, svg: &mut Vec<String>) {
        // glyph defs
        svg.push("<g>".to_owned());
        let mut g = glyph_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b: &(String, String)| a.0.cmp(&b.0));
        for (_, glyph) in g {
            svg.push(glyph);
        }
        svg.push("</g>".to_owned());
    }

    fn clip_paths(clip_paths: ClipPathMap, svg: &mut Vec<String>) {
        // clip paths
        let mut g = clip_paths.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in g {
            svg.push(format!(
                r##"<clipPath id="c{:x}"><path d="{}"/></clipPath>"##,
                id, clip_path
            ));
        }
    }

    fn render_svg(output: Arc<Document>) -> (SvgDocument, String) {
        let instant = std::time::Instant::now();

        let mut svg = vec![Self::header(&output)];
        let mut svg_body = vec![];

        // render the document
        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();

        let svg_doc = Self::svg_doc(&mut t, &output);
        let render_context = (output, svg_doc);
        t.render(&render_context, &mut svg_body).unwrap();
        let svg_doc = render_context.1;

        // attach the glyph defs, clip paths, and style defs
        svg.push("<defs>".to_owned());

        Self::glyph_defs(t.glyph_defs, &mut svg);
        Self::clip_paths(t.clip_paths, &mut svg);

        svg.push("</defs>".to_owned());

        // base style
        svg.push(r#"<style type="text/css">"#.to_owned());
        svg.push(String::from_utf8(include_bytes!("./typst.svg.css").to_vec()).unwrap());
        svg.push("</style>".to_owned());

        // style defs
        svg.push(r#"<style type="text/css">"#.to_owned());
        let mut g = t.style_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(g.into_iter().map(|v| v.1));
        svg.push("</style>".to_owned());

        // body
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript">"#.to_owned());
        svg.push(r#"<![CDATA["#.to_owned());
        svg.push(String::from_utf8(include_bytes!("./typst.svg.js").to_vec()).unwrap());
        svg.push(r#"]]>"#.to_owned());
        svg.push("</script>".to_owned());

        // close the svg
        svg.push("</svg>".to_owned());

        println!("svg render time: {:?}", instant.elapsed());
        (svg_doc, svg.join(""))
    }

    fn render_svg_incremental(prev: SvgDocument, output: Arc<Document>) -> (SvgDocument, String) {
        let instant = std::time::Instant::now();

        let mut svg = vec![Self::header(&output)];
        let mut svg_body = vec![];

        // render the document
        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();

        let svg_doc = Self::svg_doc(&mut t, &output);

        let render_context = (output, prev, svg_doc);
        t.render_diff(&render_context, &mut svg_body).unwrap();
        let svg_doc = render_context.2;

        // attach the glyph defs, clip paths, and style defs
        svg.push("<defs>".to_owned());

        svg.push("</defs>".to_owned());

        // base style
        svg.push(r#"<style type="text/css" data-typst-reuse="1">"#.to_owned());
        svg.push("</style>".to_owned());

        // incremental style
        svg.push(r#"<style type="text/css" data-typst-reuse="1">"#.to_owned());
        svg.push("</style>".to_owned());

        // body
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript" data-typst-reuse="1">"#.to_owned());
        svg.push("</script>".to_owned());

        svg.push("</svg>".to_owned());

        println!("svg render time (incremental): {:?}", instant.elapsed());
        (svg_doc, svg.join(""))
    }
}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        Ok(Self::render_svg(output).1)
    }
}

#[derive(Default)]
pub struct IncrementalSvgExporter {
    prev: Option<SvgDocument>,
}

impl IncrementalSvgExporter {
    pub fn render(&mut self, output: Arc<Document>) -> String {
        if self.prev.is_none() {
            let (next, svg) = SvgExporter::render_svg(output);
            self.prev = Some(next);
            return ["new", &svg].join(",");
        }

        let prev = self.prev.take().unwrap();
        let (next, svg) = SvgExporter::render_svg_incremental(prev, output);
        self.prev = Some(next);
        ["diff-v0", &svg].join(",")
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<Document, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<Vec<u8>> {
        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();

        let mut builder = ModuleBuilder::default();

        for page in output.pages.iter() {
            let item = t.lower(page);
            let _entry_id = builder.build(item);
        }

        let res = vec![];
        let _repr = builder.finalize();
        Ok(res)
    }
}
