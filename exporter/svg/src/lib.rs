//! Rendering into svg text or module.

// todo: https://github.com/typst/typst/pull/2740
// also 3c22c9f31914727f665ce10d6db9dac39a26eacb
// gradient pattern

// todo: https://github.com/typst/typst/pull/2610
// color export

use std::sync::Arc;

use typst::model::Document;
use typst::{diag::SourceResult, World};

use typst_ts_core::Exporter;

/// re-export the core types.
pub use typst_ts_core::font::{FontGlyphProvider, GlyphProvider, IGlyphProvider};
pub use typst_ts_core::vector::flat_ir::{
    self, FlatModule, Module, ModuleBuilder, MultiSvgDocument, SvgDocument,
};
pub use typst_ts_core::vector::{geom, ir, LowerBuilder};

pub(crate) mod utils;

/// (Text) backend of SVG export.
pub(crate) mod backend;
use backend::generate_text;
pub use backend::SvgGlyphBuilder;

/// frontend of SVG export, which provides a bunch of approaches to rendering
/// the document.
pub(crate) mod frontend;
pub use frontend::{
    DynamicLayoutSvgExporter, IncrSvgDocClient, IncrSvgDocServer, IncrementalRenderContext,
};
pub use frontend::{SvgExporter, SvgTask};

/// Useful transform for SVG Items.
pub(crate) mod transform;

#[derive(Default)]
pub struct SvgDataSelection {
    pub body: bool,
    pub defs: bool,
    pub css: bool,
    pub js: bool,
}

/// All the features that can be enabled or disabled.
pub trait ExportFeature {
    /// Whether to enable tracing.
    const ENABLE_TRACING: bool;

    /// Whether to attach debug info to svg elements.
    const SHOULD_ATTACH_DEBUG_INFO: bool;

    /// Whether to enable inlined svg.
    const ENABLE_INLINED_SVG: bool;

    /// Whether to render text element.
    /// The text elements is selectable and searchable.
    const SHOULD_RENDER_TEXT_ELEMENT: bool;

    /// Whether to use stable glyph id.
    /// If enabled, the glyph id will be stable across different svg files.
    ///
    /// A stable glyph id can help incremental font transfer (IFT).
    /// However, it is also permitted unstable if you will not use IFT.
    const USE_STABLE_GLYPH_ID: bool;

    const WITH_BUILTIN_CSS: bool;

    /// Whether to include js for interactive and responsive actions.
    /// If enabled, users can interact with the svg file.
    const WITH_RESPONSIVE_JS: bool;

    /// Also escape html entity.
    const AWARE_HTML_ENTITY: bool;
}

/// The default feature set which is used for exporting full-fledged svg.
pub struct DefaultExportFeature;
pub type DefaultSvgTask = SvgTask<DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const WITH_BUILTIN_CSS: bool = true;
    const WITH_RESPONSIVE_JS: bool = true;
    const AWARE_HTML_ENTITY: bool = true;
}

/// The feature set which is used for exporting plain svg.
pub struct SvgExportFeature;
pub type PlainSvgTask = SvgTask<SvgExportFeature>;

impl ExportFeature for SvgExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const WITH_BUILTIN_CSS: bool = true;
    const WITH_RESPONSIVE_JS: bool = false;
    const AWARE_HTML_ENTITY: bool = false;
}

/// Render SVG wrapped with html for [`Document`].
pub fn render_svg_html(output: &Document) -> String {
    type UsingExporter = SvgExporter<DefaultExportFeature>;
    let doc = UsingExporter::svg_doc(output);
    let mut svg = UsingExporter::render(&doc.module, &doc.pages, None);

    // wrap SVG with html
    let mut html: Vec<SvgText> = Vec::with_capacity(svg.len() + 3);
    html.push(r#"<!DOCTYPE html><html><head><meta charset="utf-8" /><title>"#.into());
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
    generate_text(transform::minify(html))
}

/// Render SVG for [`Document`].
pub fn render_svg(output: &Document) -> String {
    type UsingExporter = SvgExporter<SvgExportFeature>;
    let doc = UsingExporter::svg_doc(output);
    let svg_text = UsingExporter::render(&doc.module, &doc.pages, None);
    generate_text(transform::minify(svg_text))
}

pub use frontend::flat::export_module;

use crate::backend::SvgText;

impl<Feat: ExportFeature> Exporter<Document, String> for SvgExporter<Feat> {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // html wrap
        Ok(render_svg_html(&output))
    }
}

#[derive(Default)]
pub struct PureSvgExporter;

impl Exporter<Document, String> for PureSvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // html wrap
        Ok(render_svg(&output))
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<Document, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<Vec<u8>> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        export_module(UsingExporter::svg_doc(&output))
    }
}
