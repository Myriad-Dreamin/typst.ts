use std::sync::Arc;

use reflexo::typst::TypstDocument;
use reflexo_vec2svg::{
    render_svg, render_svg_html, DefaultExportFeature, ExportFeature, SvgExporter,
};
use typst::{diag::SourceResult, World};

use super::Exporter;

pub struct SvgHtmlExporter<Feat> {
    _marker: std::marker::PhantomData<Feat>,
}

impl<Feat> Default for SvgHtmlExporter<Feat> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl<Feat: ExportFeature> Exporter<TypstDocument, String> for SvgHtmlExporter<Feat> {
    fn export(&self, _world: &dyn World, output: Arc<TypstDocument>) -> SourceResult<String> {
        // html wrap
        Ok(render_svg_html::<Feat>(&output))
    }
}

#[derive(Default)]
pub struct PureSvgExporter;

impl Exporter<TypstDocument, String> for PureSvgExporter {
    fn export(&self, _world: &dyn World, output: Arc<TypstDocument>) -> SourceResult<String> {
        // html wrap
        Ok(render_svg(&output))
    }
}

#[derive(Default)]
pub struct SvgModuleExporter {}

impl Exporter<TypstDocument, Vec<u8>> for SvgModuleExporter {
    fn export(&self, _world: &dyn World, output: Arc<TypstDocument>) -> SourceResult<Vec<u8>> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        Ok(UsingExporter::svg_doc(&output).to_bytes())
    }
}
