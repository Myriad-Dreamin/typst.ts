use std::sync::Arc;

use reflexo::typst::TypstHtmlDocument;
use typst::{diag::SourceResult, World};

use crate::Exporter;

#[derive(Debug, Clone, Default)]
pub struct HtmlExporter {}

impl HtmlExporter {}

impl Exporter<TypstHtmlDocument, Vec<u8>> for HtmlExporter {
    fn export(&self, _world: &dyn World, output: Arc<TypstHtmlDocument>) -> SourceResult<Vec<u8>> {
        typst_html::html(output.as_ref()).map(|s| s.into_bytes())
    }
}
