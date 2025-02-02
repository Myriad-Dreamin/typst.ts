use std::sync::Arc;

use reflexo::typst::TypstDocument;
use typst::{diag::SourceResult, World};
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

use crate::{Exporter, TypstDatetime};

#[derive(Debug, Clone, Default)]
pub struct PdfDocExporter {
    ctime: Option<TypstDatetime>,
    standards: Option<PdfStandards>,
}

impl PdfDocExporter {
    pub fn with_ctime(mut self, v: Option<TypstDatetime>) -> Self {
        self.ctime = v;
        self
    }

    pub fn with_standard(mut self, v: Option<PdfStandard>) -> Self {
        self.standards = v.map(|v| PdfStandards::new(&[v]).unwrap());
        self
    }
}

impl Exporter<TypstDocument, Vec<u8>> for PdfDocExporter {
    fn export(&self, _world: &dyn World, output: Arc<TypstDocument>) -> SourceResult<Vec<u8>> {
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
