use std::sync::Arc;

use reflexo::typst::TypstPagedDocument;
use typst::{diag::SourceResult, World};
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards, Timestamp};

use crate::Exporter;

#[derive(Debug, Clone, Default)]
pub struct PdfDocExporter {
    ctime: Option<Timestamp>,
    standards: Option<PdfStandards>,
}

impl PdfDocExporter {
    pub fn with_ctime(mut self, v: Option<Timestamp>) -> Self {
        self.ctime = v;
        self
    }

    pub fn with_standard(mut self, v: Option<PdfStandard>) -> Self {
        self.standards = v.map(|v| PdfStandards::new(&[v]).unwrap());
        self
    }
}

impl Exporter<TypstPagedDocument, Vec<u8>> for PdfDocExporter {
    fn export(&self, _world: &dyn World, output: Arc<TypstPagedDocument>) -> SourceResult<Vec<u8>> {
        // todo: ident option

        typst_pdf::pdf(
            output.as_ref(),
            &PdfOptions {
                timestamp: self.ctime,
                standards: self.standards.clone().unwrap_or_default(),
                ..Default::default()
            },
        )
    }
}
