use std::sync::Arc;

pub use typst_pdf::pdf;
use typst_ts_core::{Exporter, TypstDatetime};

use typst::{diag::SourceResult, foundations::Smart, World};

#[derive(Debug, Clone, Default)]
pub struct PdfDocExporter {
    ctime: Option<TypstDatetime>,
}

impl PdfDocExporter {
    pub fn with_ctime(mut self, v: Option<TypstDatetime>) -> Self {
        self.ctime = v;
        self
    }
}

impl Exporter<typst::model::Document, Vec<u8>> for PdfDocExporter {
    fn export(
        &self,
        _world: &dyn World,
        output: Arc<typst::model::Document>,
    ) -> SourceResult<Vec<u8>> {
        // todo: ident option

        Ok(typst_pdf::pdf(output.as_ref(), Smart::Auto, self.ctime))
    }
}
