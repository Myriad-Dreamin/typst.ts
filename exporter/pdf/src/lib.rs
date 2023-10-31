use std::sync::Arc;

pub use typst::export::pdf;
use typst_ts_core::Exporter;

use typst::{diag::SourceResult, World};

#[derive(Debug, Clone, Default)]
pub struct PdfDocExporter {}

impl Exporter<typst::doc::Document, Vec<u8>> for PdfDocExporter {
    fn export(
        &self,
        _world: &dyn World,
        output: Arc<typst::doc::Document>,
    ) -> SourceResult<Vec<u8>> {
        // todo: ident and timestamp option
        Ok(typst::export::pdf(output.as_ref(), None, None))
    }
}
