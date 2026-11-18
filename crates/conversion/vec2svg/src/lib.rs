//! Rendering into svg text or module.

// todo: https://github.com/typst/typst/pull/2610
// color export

use reflexo::{
    escape::{escape_str, PcDataEscapes},
    typst::{TypstDocumentTrait, TypstPagedDocument},
};

/// re-export the core types.
pub use reflexo_typst2vec::font::{FontGlyphProvider, GlyphProvider, IGlyphProvider};
pub use reflexo_typst2vec::ir::{self, geom, FlatModule, Module, MultiVecDocument, VecDocument};

pub(crate) mod utils;

/// (Text) backend of SVG export.
pub(crate) mod backend;
use backend::generate_text;
pub use backend::SvgGlyphBuilder;
pub use backend::SvgText;

/// frontend of SVG export, which provides a bunch of approaches to rendering
/// the document.
pub(crate) mod frontend;
pub use frontend::{
    svg_gradient_def, svg_gradient_def_ref, GradientDefRef, SVGGradientDef, SvgExporter, SvgTask,
};
pub use frontend::{
    DynamicLayoutSvgExporter, IncrSvgDocClient, IncrSvgDocServer, IncrementalRenderContext,
};

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
pub trait ExportFeature: Send + Sync + 'static {
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

    /// Whether to rasterize text.
    const SHOULD_RASTERIZE_TEXT: bool;

    /// Whether to include builtin css.
    const WITH_BUILTIN_CSS: bool;

    /// Whether to include js for interactive and responsive actions.
    /// If enabled, users can interact with the svg file.
    const WITH_RESPONSIVE_JS: bool;

    /// Also escape html entity.
    const AWARE_HTML_ENTITY: bool;
}

/// The default feature set which is used for exporting full-fledged svg.
pub struct DefaultExportFeature;
pub type DefaultSvgTask = SvgTask<'static, DefaultExportFeature>;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const SHOULD_RASTERIZE_TEXT: bool = false;
    const WITH_BUILTIN_CSS: bool = true;
    const WITH_RESPONSIVE_JS: bool = true;
    const AWARE_HTML_ENTITY: bool = true;
}

/// The feature set which is used for exporting plain svg.
pub struct SvgExportFeature;
pub type PlainSvgTask = SvgTask<'static, SvgExportFeature>;

impl ExportFeature for SvgExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const SHOULD_RASTERIZE_TEXT: bool = false;
    const WITH_BUILTIN_CSS: bool = true;
    const WITH_RESPONSIVE_JS: bool = false;
    const AWARE_HTML_ENTITY: bool = false;
}

/// Render SVG wrapped with html for [`TypstPagedDocument`].
pub fn render_svg_html<Feat: ExportFeature>(output: &TypstPagedDocument) -> String {
    let mut doc = SvgExporter::<Feat>::svg_doc(output);
    doc.module.prepare_glyphs();
    let mut svg = SvgExporter::<Feat>::render(&doc.module, &doc.pages, None);

    // wrap SVG with html
    let mut html: Vec<SvgText> = Vec::with_capacity(svg.len() + 3);
    html.push(r#"<!DOCTYPE html><html><head><meta charset="utf-8" /><title>"#.into());
    html.push(SvgText::Plain(
        output
            .info()
            .title
            .as_ref()
            .map(|s| escape_str::<PcDataEscapes>(s).into_owned())
            .unwrap_or_else(|| "Typst TypstPagedDocument".into()),
    ));
    html.push(r#"</title></head><body>"#.into());
    html.append(&mut svg);
    html.push(r#"</body></html>"#.into());
    generate_text(transform::minify(html))
}

/// Render SVG for [`TypstPagedDocument`].
pub fn render_svg(output: &TypstPagedDocument) -> String {
    type UsingExporter = SvgExporter<SvgExportFeature>;
    let mut doc = UsingExporter::svg_doc(output);
    doc.module.prepare_glyphs();
    let svg_text = UsingExporter::render(&doc.module, &doc.pages, None);
    generate_text(transform::minify(svg_text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reflexo::{
        hash::Fingerprint,
        vector::ir::{Axes, LinkItem, Module, Page, Scalar, VecItem},
    };

    #[test]
    fn html_title_is_escaped_as_text() {
        let mut info = typst::model::DocumentInfo::default();
        info.title = Some("</title><script>alert(1)</script>&".into());
        let output = TypstPagedDocument::new(Default::default(), info);

        let html = render_svg_html::<SvgExportFeature>(&output);

        assert!(
            html.contains("<title>&lt;/title>&lt;script>alert(1)&lt;/script>&amp;</title>"),
            "{html}"
        );
        assert!(!html.contains("<script>alert(1)</script>"), "{html}");
    }

    #[test]
    fn authored_typst_href_is_not_rendered_as_svg_code() {
        let link_id = Fingerprint::from_pair(1, 2);
        let size = Axes::new(Scalar(10.0), Scalar(10.0));
        let link = LinkItem {
            href: "@typst:custom&route".into(),
            size,
        };
        let mut module = Module::default();
        module.items.insert(link_id, VecItem::Link(link));
        let page = Page {
            content: link_id,
            size,
        };

        let svg = SvgExporter::<SvgExportFeature>::render_flat_svg(&module, &[page], None);

        assert!(!svg.contains("onclick="), "{svg}");
        assert!(
            svg.contains("xlink:href=\"@typst:custom&amp;route\""),
            "{svg}"
        );
    }

    #[test]
    fn generated_typst_location_keeps_svg_navigation() {
        let link_id = Fingerprint::from_pair(1, 2);
        let size = Axes::new(Scalar(10.0), Scalar(10.0));
        let link = LinkItem {
            href: "@typst:handleTypstLocation(this, 2, 3.5, 4.25)".into(),
            size,
        };
        let mut module = Module::default();
        module.items.insert(link_id, VecItem::Link(link));
        let page = Page {
            content: link_id,
            size,
        };

        let svg = SvgExporter::<SvgExportFeature>::render_flat_svg(&module, &[page], None);

        assert!(
            svg.contains("onclick=\"handleTypstLocation(this, 2, 3.5, 4.25); return false\""),
            "{svg}"
        );
    }
}
