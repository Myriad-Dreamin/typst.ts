use std::sync::Arc;

pub use typst_pdf::pdf;
use typst_ts_core::Exporter;

use typst::{diag::SourceResult, World};

#[derive(Debug, Clone, Default)]
pub struct PdfDocExporter {
    with_timestamp: bool,
}

impl PdfDocExporter {
    pub fn with_timestamp(mut self, enable: bool) -> Self {
        self.with_timestamp = enable;
        self
    }
}

impl Exporter<typst::model::Document, Vec<u8>> for PdfDocExporter {
    fn export(
        &self,
        world: &dyn World,
        output: Arc<typst::model::Document>,
    ) -> SourceResult<Vec<u8>> {
        // todo: ident option

        let timestamp = self.with_timestamp.then(|| world.today(None)).flatten();
        Ok(typst_pdf::pdf(output.as_ref(), None, timestamp))
    }
}
