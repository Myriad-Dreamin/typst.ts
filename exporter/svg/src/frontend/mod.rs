use std::sync::Arc;

use typst::doc::Document;
use typst_ts_core::{
    font::GlyphProvider,
    hash::FingerprintBuilder,
    vector::{
        flat_ir::{self, Module},
        ir::{AbsoluteRef, Axes, GlyphItem, GlyphMapping, GlyphPackBuilder, Size, SvgItem},
        vm::RenderVm,
        LowerBuilder,
    },
};

pub(crate) mod context;
use context::{ClipPathMap, RenderContext, StyleDefMap};

#[cfg(feature = "flat-vector")]
pub(crate) mod dynamic_layout;
pub use dynamic_layout::DynamicLayoutSvgExporter;
#[cfg(feature = "flat-vector")]
pub(crate) mod flat;
#[cfg(feature = "flat-vector")]
pub(crate) mod incremental;
use crate::{
    backend::{SvgGlyphBuilder, SvgText, SvgTextNode},
    utils::AbsExt,
    ExportFeature,
};
pub use incremental::{IncrSvgDocClient, IncrSvgDocServer, IncrementalRenderContext};

pub struct SvgExporter<Feat: ExportFeature> {
    pub _feat_phantom: std::marker::PhantomData<Feat>,
}

impl<Feat: ExportFeature> Default for SvgExporter<Feat> {
    fn default() -> Self {
        Self {
            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgExporter<Feat> {
    /// Render the header of SVG.
    /// <svg> .. </svg>
    /// ^^^^^
    fn header_inner(w: f32, h: f32) -> String {
        format!(
            r#"<svg class="typst-doc" viewBox="0 0 {:.3} {:.3}" width="{:.3}" height="{:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            w, h, w, h,
        )
    }

    /// Render the header of SVG for [`Document`].
    /// <svg> .. </svg>
    /// ^^^^^
    fn header_doc(output: &Document) -> String {
        // calculate the width and height of SVG
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

    /// Render the style for SVG
    /// <svg> <style/> .. </svg>
    ///       ^^^^^^^^
    /// See [`StyleDefMap`].
    fn style_defs(style_defs: StyleDefMap, svg: &mut Vec<SvgText>) {
        // style defs
        svg.push(r#"<style type="text/css">"#.into());

        // sort and push the style defs
        let mut style_defs = style_defs.into_iter().collect::<Vec<_>>();
        style_defs.sort_by(|a, b| a.0.cmp(&b.0));
        svg.extend(style_defs.into_iter().map(|v| SvgText::Plain(v.1)));

        svg.push("</style>".into());
    }

    /// Render the clip paths for SVG
    /// <svg> <defs> <clipPath/> </defs> .. </svg>
    ///              ^^^^^^^^^^^
    /// See [`ClipPathMap`].
    fn clip_paths(clip_paths: ClipPathMap, svg: &mut Vec<SvgText>) {
        let mut clip_paths = clip_paths.into_iter().collect::<Vec<_>>();
        clip_paths.sort_by(|a, b| a.1.cmp(&b.1));
        for (clip_path, id) in clip_paths {
            svg.push(SvgText::Plain(format!(
                r##"<clipPath id="{}"><path d="{}"/></clipPath>"##,
                id.as_svg_id("c"),
                clip_path
            )));
        }
    }

    /// Template SVG.
    fn render_svg_template(
        t: SvgTask<Feat>,
        header: String,
        mut body: Vec<SvgText>,
        glyphs: impl IntoIterator<Item = SvgText>,
    ) -> Vec<SvgText> {
        let mut svg = vec![
            SvgText::Plain(header),
            // base style
        ];

        if Feat::WITH_BUILTIN_CSS {
            svg.push(r#"<style type="text/css">"#.into());
            svg.push(include_str!("./typst.svg.css").into());
            svg.push("</style>".into());
        }

        // attach the glyph defs, clip paths, and style defs
        svg.push(r#"<defs class="glyph">"#.into());
        svg.extend(glyphs);
        svg.push("</defs>".into());
        svg.push(r#"<defs class="clip-path">"#.into());
        Self::clip_paths(t.clip_paths, &mut svg);
        svg.push("</defs>".into());
        Self::style_defs(t.style_defs, &mut svg);

        // body
        svg.append(&mut body);

        if Feat::WITH_RESPONSIVE_JS {
            // attach the javascript for animations
            svg.push(r#"<script type="text/javascript">"#.into());
            svg.push(include_str!("./typst.svg.js").into());
            svg.push("</script>".into());
        }

        // close SVG
        svg.push("</svg>".into());

        svg
    }

    /// Render SVG for [`Document`].
    /// It does not flatten the vector items before rendering so called
    /// "transient".
    pub(crate) fn render_transient_svg(output: &Document) -> Vec<SvgText> {
        let mut t = SvgTask::<Feat>::default();

        // render SVG header
        let header = Self::header_doc(output);

        // lowering the document into svg items
        let mut lower_builder = LowerBuilder::new(output);
        let pages = output
            .pages
            .iter()
            .map(|p| lower_builder.lower(p))
            .collect::<Vec<_>>();

        // render SVG body
        let mut svg_body = vec![];
        t.render_pages_transient(output, pages, &mut svg_body);

        // render the glyphs collected from the pages
        let glyphs = GlyphPackBuilder::finalize(std::mem::take(&mut t.glyph_defs));
        let glyphs = t.render_glyphs(glyphs.iter().map(|(x, y)| (x, y)), false);

        // template SVG
        Self::render_svg_template(t, header, svg_body, glyphs)
    }

    /// Render SVG wrapped with HTML for [`Document`].
    /// It does not flatten the vector items before rendering so called
    /// "transient".
    pub(crate) fn render_transient_html(output: &Document) -> Vec<SvgText> {
        // render SVG
        let mut svg = Self::render_transient_svg(output);

        // wrap SVG with html
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

/// The task context for exporting svg.
/// It is also as a namespace for all the functions used in the task.
pub struct SvgTask<Feat: ExportFeature> {
    /// Provides glyphs.
    /// See [`GlyphProvider`].
    glyph_provider: GlyphProvider,

    /// A fingerprint builder for generating unique id.
    fingerprint_builder: FingerprintBuilder,

    /// Stores the glyphs used in the document.
    pub(crate) glyph_defs: GlyphMapping,
    /// Stores the style definitions used in the document.
    pub(crate) style_defs: StyleDefMap,
    /// Stores the clip paths used in the document.
    pub(crate) clip_paths: ClipPathMap,

    _feat_phantom: std::marker::PhantomData<Feat>,
}

/// Unfortunately, `Default` derive does not work for generic structs.
impl<Feat: ExportFeature> Default for SvgTask<Feat> {
    fn default() -> Self {
        Self {
            glyph_provider: GlyphProvider::default(),

            fingerprint_builder: FingerprintBuilder::default(),

            glyph_defs: GlyphMapping::default(),
            style_defs: StyleDefMap::default(),
            clip_paths: ClipPathMap::default(),

            _feat_phantom: std::marker::PhantomData,
        }
    }
}

impl<Feat: ExportFeature> SvgTask<Feat> {
    /// Sets the glyph provider for task.
    pub fn set_glyph_provider(&mut self, glyph_provider: GlyphProvider) {
        self.glyph_provider = glyph_provider;
    }

    /// Return integral page size for showing document.
    pub(crate) fn page_size(sz: Size) -> Axes<u32> {
        let (width_px, height_px) = {
            let width_px = (sz.x.0.ceil()).round().max(1.0) as u32;
            let height_px = (sz.y.0.ceil()).round().max(1.0) as u32;

            (width_px, height_px)
        };

        Axes::new(width_px, height_px)
    }

    /// fork a render task with module.
    #[cfg(feature = "flat-vector")]
    pub fn get_render_context<'m, 't>(
        &'t mut self,
        module: &'m flat_ir::Module,
    ) -> RenderContext<'m, 't, Feat> {
        RenderContext::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            module,

            fingerprint_builder: &mut self.fingerprint_builder,

            glyph_defs: &mut self.glyph_defs,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_attach_debug_info: Feat::SHOULD_ATTACH_DEBUG_INFO,
            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
        }
    }

    /// fork a render task.
    #[cfg(not(feature = "flat-vector"))]
    pub fn get_render_context<'m>(&mut self) -> RenderContext<'m, '_, Feat> {
        RenderContext::<Feat> {
            glyph_provider: self.glyph_provider.clone(),

            fingerprint_builder: &mut self.fingerprint_builder,

            glyph_defs: &mut self.glyph_defs,
            style_defs: &mut self.style_defs,
            clip_paths: &mut self.clip_paths,

            should_attach_debug_info: Feat::SHOULD_ATTACH_DEBUG_INFO,
            should_render_text_element: true,
            use_stable_glyph_id: true,

            _feat_phantom: Default::default(),
            _m_phantom: Default::default(),
        }
    }

    /// Render glyphs into the svg_body.
    pub(crate) fn render_glyphs<'a, I: Iterator<Item = (&'a AbsoluteRef, &'a GlyphItem)>>(
        &mut self,
        glyphs: I,
        use_stable_glyph_id: bool,
    ) -> Vec<SvgText> {
        let mut render_task = SvgGlyphBuilder {
            glyph_provider: self.glyph_provider.clone(),
        };

        let mut svg_body = Vec::new();

        for (abs_ref, item) in glyphs {
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

    /// Render pages into the svg_body.
    pub fn render_pages_transient(
        &mut self,
        output: &Document,
        pages: Vec<SvgItem>,
        svg_body: &mut Vec<SvgText>,
    ) {
        #[cfg(feature = "flat-vector")]
        let module = Module::default();
        let mut render_task = {
            #[cfg(feature = "flat-vector")]
            let render_task = self.get_render_context(&module);

            #[cfg(not(feature = "flat-vector"))]
            let render_task = self.get_render_context();

            render_task
        };

        render_task.use_stable_glyph_id = false;

        // accumulate the height of pages
        let mut acc_height = 0u32;
        for (idx, page) in pages.iter().enumerate() {
            let size = Self::page_size(output.pages[idx].size().into());

            let attributes = vec![
                ("transform", format!("translate(0, {})", acc_height)),
                ("data-page-width", size.x.to_string()),
                ("data-page-height", size.y.to_string()),
            ];

            let page_svg = render_task.render_item(page);

            svg_body.push(SvgText::Content(Arc::new(SvgTextNode {
                attributes,
                content: vec![SvgText::Content(page_svg)],
            })));
            acc_height += size.y;
        }
    }
}
