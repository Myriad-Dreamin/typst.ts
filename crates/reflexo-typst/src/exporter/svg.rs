use reflexo::typst::Bytes;
use reflexo::typst::TypstPagedDocument;
use reflexo_vec2svg::{render_svg, render_svg_html, ExportFeature, SvgExporter};
use serde::{Deserialize, Serialize};
use tinymist_task::{ExportSvgTask, ExportTask};

use super::prelude::*;

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExportWebSvgModuleTask {
    #[serde(flatten)]
    pub export: ExportTask,
}

pub struct WebSvgModuleExport<EF>(std::marker::PhantomData<EF>);

impl<EF: ExportFeature, F: CompilerFeat> ExportComputation<F, TypstPagedDocument>
    for WebSvgModuleExport<EF>
{
    type Output = Bytes;
    type Config = ExportWebSvgModuleTask;

    fn run(
        _g: &Arc<WorldComputeGraph<F>>,
        doc: &Arc<TypstPagedDocument>,
        _config: &Self::Config,
    ) -> Result<Bytes> {
        Ok(Bytes::new(SvgExporter::<EF>::svg_doc(doc).to_bytes()))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExportWebSvgTask {
    #[serde(flatten)]
    pub base: ExportSvgTask,
}

pub struct WebSvgExport<EF>(std::marker::PhantomData<EF>);

impl<EF: ExportFeature, F: CompilerFeat> ExportComputation<F, TypstPagedDocument>
    for WebSvgExport<EF>
{
    type Output = String;
    type Config = ExportWebSvgTask;

    fn run(
        _g: &Arc<WorldComputeGraph<F>>,
        doc: &Arc<TypstPagedDocument>,
        _config: &Self::Config,
    ) -> Result<String> {
        Ok(render_svg(doc))
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ExportWebSvgHtmlTask {
    #[serde(flatten)]
    pub base: ExportSvgTask,
}

pub struct WebSvgHtmlExport<EF>(std::marker::PhantomData<EF>);

impl<EF: ExportFeature, F: CompilerFeat> ExportComputation<F, TypstPagedDocument>
    for WebSvgHtmlExport<EF>
{
    type Output = String;
    type Config = ExportWebSvgHtmlTask;

    fn run(
        _g: &Arc<WorldComputeGraph<F>>,
        doc: &Arc<TypstPagedDocument>,
        _config: &Self::Config,
    ) -> Result<String> {
        Ok(render_svg_html::<EF>(doc))
    }
}
