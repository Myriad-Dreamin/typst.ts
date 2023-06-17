//! Rendering into svg text or module.

pub(crate) use tiny_skia as sk;

use std::collections::HashMap;
use std::sync::Arc;

use typst::diag::SourceResult;
use typst::doc::Document;
use typst::World;

use typst_ts_core::font::GlyphProvider;
use typst_ts_core::Exporter;

pub(crate) mod escape;
#[cfg(feature = "flat-vector")]
pub(crate) mod flat_vector;
pub(crate) mod path2d;
pub(crate) mod render;
pub(crate) mod utils;
pub(crate) mod vector;

use crate::vector::codegen::generate_text;
use ir::{ImmutStr, StyleNs, SvgItem};
use vector::*;
pub use vector::{geom, ir};

use render::GlyphRenderTask;
#[cfg(feature = "flat-vector")]
pub use render::{
    dynamic_layout::DynamicLayoutSvgExporter,
    flat::{serialize_multi_doc_standalone, SvgModuleExporter},
    incremental::IncrementalSvgExporter,
};

#[cfg(feature = "flat-vector")]
pub use flat_ir::{
    LayoutElem, Module, ModuleBuilder, MultiSvgDocument, Pages, SerializedModule, SvgDocument,
};
#[cfg(feature = "flat-vector")]
pub use flat_vector::ir as flat_ir;

use utils::AbsExt;

pub trait ExportFeature {
    const ENABLE_TRACING: bool;
    const SHOULD_RENDER_TEXT_ELEMENT: bool;
    const USE_STABLE_GLYPH_ID: bool;
}

pub struct DefaultExportFeature;
pub type DefaultSvgTask = SvgTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
}

type StyleDefMap = HashMap<(StyleNs, ImmutStr), String>;
type ClipPathMap = HashMap<ImmutStr, u32>;

pub struct SvgTask<Feat: ExportFeature = DefaultExportFeature> {
    glyph_provider: GlyphProvider,

    incr: GlyphPackBuilder,
    style_defs: StyleDefMap,
    clip_paths: ClipPathMap,

    _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: ExportFeature> Default for SvgTask<Feat> {
    fn default() -> Self {
        Self {
            glyph_provider: GlyphProvider::default(),

            incr: GlyphPackBuilder::default(),
            style_defs: StyleDefMap::default(),
            clip_paths: ClipPathMap::default(),

            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgTask<Feat> {
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

    #[cfg(feature = "flat-vector")]
    pub fn fork_render_task<'m, 't>(
        &'t mut self,
        module: &'m flat_ir::Module,
    ) -> SvgRenderTask<'m, 't, DefaultExportFeature> {
        SvgRenderTask::<DefaultExportFeature> {
            glyph_provider: self.glyph_provider.clone(),

            page_off: 0,

            module,

            incr: &mut self.incr,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }

    #[cfg(not(feature = "flat-vector"))]
    pub fn fork_render_task<'m, 't>(&'t mut self) -> SvgRenderTask<'m, 't, DefaultExportFeature> {
        SvgRenderTask::<DefaultExportFeature> {
            glyph_provider: self.glyph_provider.clone(),

            page_off: 0,

            incr: &mut self.incr,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
            _m_phantom: Default::default(),
        }
    }

    pub fn fork_glyph_render_task(&self) -> GlyphRenderTask {
        GlyphRenderTask {
            glyph_provider: self.glyph_provider.clone(),
        }
    }

    /// Render a document into the svg_body.
    fn render_glyphs(&mut self, glyphs: &GlyphPack, use_stable_glyph_id: bool) -> Vec<SvgText> {
        let mut render_task = self.fork_glyph_render_task();

        let mut svg_body = Vec::new();

        for (abs_ref, item) in glyphs.iter() {
            let glyph_id = if Feat::USE_STABLE_GLYPH_ID && use_stable_glyph_id {
                abs_ref.as_svg_id("g")
            } else {
                abs_ref.as_unstable_svg_id("g")
            };
            svg_body.push(SvgText::Plain(
                render_task
                    .render_glyph(&glyph_id, item)
                    .unwrap_or_default(),
            ))
        }

        svg_body
    }

    /// Render a document into the svg_body.
    pub fn render_transient(
        &mut self,
        output: &Document,
        pages: Vec<SvgItem>,
        svg_body: &mut Vec<SvgText>,
    ) {
        #[cfg(feature = "flat-vector")]
        let module = Module::default();
        let mut render_task = {
            #[cfg(feature = "flat-vector")]
            let render_task = self.fork_render_task(&module);

            #[cfg(not(feature = "flat-vector"))]
            let render_task = self.fork_render_task();

            render_task
        };

        render_task.use_stable_glyph_id = false;

        let mut acc_height = 0u32;
        for (idx, page) in pages.iter().enumerate() {
            render_task.page_off = idx;

            let size = Self::page_size(output.pages[idx].size().into());

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes: vec![
                    ("transform", format!("translate(0, {})", acc_height)),
                    ("data-page-width", size.x.to_string()),
                    ("data-page-height", size.y.to_string()),
                ],
                content: vec![SvgText::Content(render_task.render_item(page))],
            })));
            acc_height += size.y;
        }
    }
}

use crate::ir::GlyphPackBuilder;

pub struct SvgRenderTask<'m, 't, Feat: ExportFeature = DefaultExportFeature> {
    pub glyph_provider: GlyphProvider,

    #[cfg(feature = "flat-vector")]
    pub module: &'m Module,
    pub incr: &'t mut GlyphPackBuilder,

    pub style_defs: &'t mut StyleDefMap,
    pub clip_paths: &'t mut ClipPathMap,

    pub page_off: usize,
    pub should_render_text_element: bool,
    pub use_stable_glyph_id: bool,

    pub _feat_phantom: std::marker::PhantomData<Feat>,
    #[cfg(not(feature = "flat-vector"))]
    pub _m_phantom: std::marker::PhantomData<&'m ()>,
}

#[derive(Default)]
pub struct SvgExporter {}

impl SvgExporter {
    fn header_inner(w: f32, h: f32) -> String {
        format!(
            r#"<svg class="typst-doc" viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        )
    }

    fn header_doc(output: &Document) -> String {
        // calculate the width and height of the svg
        let w = output
            .pages
            .iter()
            .map(|p| p.size().x.to_f32().ceil())
            .max_by(|a, b| a.total_cmp(b))
            .unwrap();
        let h = output
            .pages
            .iter()
            .map(|p| p.size().y.to_f32().ceil())
            .sum::<f32>();

        Self::header_inner(w, h)
    }

    fn style_defs(style_defs: StyleDefMap, svg: &mut Vec<SvgText>) {
        // style defs
        svg.push(r#"<style type="text/css">"#.into());
        let mut g = style_defs.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(g.into_iter().map(|v| SvgText::Plain(v.1)));
        svg.push("</style>".into());
    }

    fn clip_paths(clip_paths: ClipPathMap, svg: &mut Vec<SvgText>) {
        let mut g = clip_paths.into_iter().collect::<Vec<_>>();
        g.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in g {
            svg.push(SvgText::Plain(format!(
                r##"<clipPath id="c{:x}"><path d="{}"/></clipPath>"##,
                id, clip_path
            )));
        }
    }

    fn render_template(
        t: SvgTask,
        header: String,
        mut body: Vec<SvgText>,
        glyphs: impl IntoIterator<Item = SvgText>,
    ) -> Vec<SvgText> {
        let mut svg = vec![
            SvgText::Plain(header),
            // base style
            r#"<style type="text/css">"#.into(),
            include_str!("./typst.svg.css").into(),
            "</style>".into(),
            // attach the glyph defs, clip paths, and style defs
            "<defs>".into(),
            "<g>".into(),
        ];
        svg.extend(glyphs);
        svg.push("</g>".into());
        Self::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".into());
        Self::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut body);

        // attach the javascript for animations
        svg.push(r#"<script type="text/javascript">"#.into());
        svg.push(include_str!("./typst.svg.js").into());
        svg.push("</script>".into());

        // close the svg
        svg.push("</svg>".into());

        svg
    }

    fn render_transient_svg(output: &Document) -> Vec<SvgText> {
        let instant = std::time::Instant::now();
        let header = Self::header_doc(output);

        let mut lower_builder = LowerBuilder::new(output);
        let pages = output
            .pages
            .iter()
            .map(|p| lower_builder.lower(p))
            .collect::<Vec<_>>();

        let mut t = SvgTask::<DefaultExportFeature>::default();
        let mut svg_body = vec![];
        t.render_transient(output, pages, &mut svg_body);

        let (module, ..) = std::mem::take(&mut t.incr).finalize();
        let glyphs = t.render_glyphs(&module, false);

        let svg = Self::render_template(t, header, svg_body, glyphs.into_iter());
        println!("svg render time: {:?}", instant.elapsed());
        svg
    }

    fn render_transient_html(output: &Document) -> Vec<SvgText> {
        let mut svg = Self::render_transient_svg(output);
        let mut html: Vec<SvgText> = Vec::with_capacity(svg.len() + 3);
        html.push(r#"<html><head><meta charset="utf-8" /><title>"#.into());
        html.push(SvgText::Plain(
            output
                .title
                .clone()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Typst Document".into()),
        ));
        html.push(r#"</title></head><body>"#.into());
        html.append(&mut svg);
        html.push(r#"</body></html>"#.into());

        html
    }
}

impl Exporter<Document, String> for SvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // html wrap
        Ok(generate_text(Self::render_transient_html(&output)))
    }
}

pub fn export(output: &Document) -> String {
    generate_text(SvgExporter::render_transient_html(output))
}
