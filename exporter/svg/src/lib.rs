//! Rendering into svg text or module.

use std::collections::HashMap;
use std::sync::Arc;

use ir::{ModuleBuilder, StyleNs};
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

pub struct SvgTask<Feat: RenderFeature = DefaultRenderFeature> {
    glyph_provider: GlyphProvider,
    annotation_proc: AnnotationProcessor,

    style_defs: HashMap<(StyleNs, Arc<str>), String>,
    glyph_defs: HashMap<String, String>,
    clip_paths: HashMap<Arc<str>, u32>,

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

    /// Render a document into the svg_body.
    pub fn render(&mut self, input: Arc<Document>, svg_body: &mut Vec<String>) -> ZResult<()> {
        let mut acc_height = 0f32;
        for (idx, page) in input.pages.iter().enumerate() {
            let (item, size) = self.render_frame(idx, page).unwrap();
            let item = format!(
                r#"<g transform="translate(0, {})" >{}</g>"#,
                acc_height, item
            );

            svg_body.push(item);
            acc_height += size.y as f32;
        }

        Ok(())
    }

    /// Render a frame into the a `<g/>` element.
    pub fn render_frame(&mut self, idx: usize, frame: &Frame) -> ZResult<(String, Axes<u32>)> {
        let item = self.lower(frame);
        let (entry, module) = item.flatten();

        let (width_px, height_px) = {
            let size = frame.size();
            let width_px = (size.x.to_pt().ceil() as f32).round().max(1.0) as u32;
            let height_px = (size.y.to_pt().ceil() as f32).round().max(1.0) as u32;

            (width_px, height_px)
        };

        let default_glyph_provider = GlyphProvider::new(FontGlyphProvider::default());

        let mut t = SvgRenderTask::<DefaultRenderFeature> {
            glyph_provider: default_glyph_provider,

            module: &module,

            style_defs: &mut self.style_defs,
            glyph_defs: &mut self.glyph_defs,
            clip_paths: &mut self.clip_paths,
            render_text_element: true,

            page_off: idx,
            width_px,
            height_px,
            raw_height: height_px as f32,

            font_map: HashMap::default(),

            _feat_phantom: Default::default(),
        };

        Ok((t.render_item(entry)?, Axes::new(width_px, height_px)))
    }
}

#[derive(Default)]
pub struct SvgExporter {}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // todo: without page
        // calculate the width and height of the svg
        let w = output.pages[0].width().to_pt().ceil();
        let h = output.pages[0].height().to_pt().ceil() * output.pages.len() as f64;

        let header = format!(
            r#"<svg viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        );
        let mut svg = vec![header];
        let mut svg_body = vec![];

        // render the document
        let mut t = SvgTask::<DefaultRenderFeature>::new(&output).unwrap();
        t.render(output, &mut svg_body).unwrap();

        // attach the glyph defs, clip paths, and style defs
        svg.push("<defs>".to_owned());

        // glyph defs
        svg.push("<g>".to_owned());
        let mut g = t.glyph_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, glyph) in g {
            svg.push(glyph);
        }
        svg.push("</g>".to_owned());

        // clip paths
        let mut g = t.clip_paths.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in g {
            svg.push(format!(
                r##"<clipPath id="c{:x}"><path d="{}"/></clipPath>"##,
                id, clip_path
            ));
        }

        svg.push("</defs>".to_owned());

        // style defs
        svg.push(r#"<style type="text/css">"#.to_owned());
        svg.push(String::from_utf8(include_bytes!("./typst.svg.css").to_vec()).unwrap());
        let mut g = t.style_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(g.into_iter().map(|v| v.1));
        svg.push("</style>".to_owned());
        svg.append(&mut svg_body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript">"#.to_owned());
        svg.push(r#"<![CDATA["#.to_owned());
        svg.push(String::from_utf8(include_bytes!("./typst.svg.js").to_vec()).unwrap());
        svg.push(r#"]]>"#.to_owned());
        svg.push("</script>".to_owned());

        // close the svg
        svg.push("</svg>".to_owned());

        Ok(svg.join(""))
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
