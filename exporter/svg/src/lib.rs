//! Rendering into svg text or module.

use std::sync::Arc;

use typst::doc::Document;
use typst::{diag::SourceResult, World};

use typst_ts_core::Exporter;

/// re-export the core types.
pub use typst_ts_core::font::{FontGlyphProvider, GlyphProvider, IGlyphProvider};
#[cfg(feature = "flat-vector")]
pub use typst_ts_core::vector::flat_ir::{
    self, LayoutElem, Module, ModuleBuilder, MultiSvgDocument, Pages, SerializedModule, SvgDocument,
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
#[cfg(feature = "flat-vector")]
pub use frontend::{DynamicLayoutSvgExporter, IncrementalRenderContext, IncrementalSvgExporter};
pub use frontend::{SvgExporter, SvgTask};

/// Useful transform for SVG Items.
pub(crate) mod transform;

/// All the features that can be enabled or disabled.
pub trait ExportFeature {
    /// Whether to enable tracing.
    const ENABLE_TRACING: bool;

    /// Whether to attach debug info to svg elements.
    const SHOULD_ATTACH_DEBUG_INFO: bool;

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
    let svg_text = UsingExporter::render_transient_html(output);
    generate_text(transform::minify(svg_text))
}

/// Render SVG for [`Document`].
pub fn render_svg(output: &Document) -> String {
    type UsingExporter = SvgExporter<SvgExportFeature>;
    let svg_text = UsingExporter::render_transient_html(output);
    generate_text(transform::minify(svg_text))
}

#[cfg(feature = "flat-vector")]
pub use frontend::flat::export_module;

impl<Feat: ExportFeature> Exporter<Document, String> for SvgExporter<Feat> {
    fn export(&self, _world: &dyn World, output: Arc<Document>) -> SourceResult<String> {
        // html wrap
        Ok(render_svg_html(&output))
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

        Ok(flat_ir::serialize_module(repr))
    }
}
