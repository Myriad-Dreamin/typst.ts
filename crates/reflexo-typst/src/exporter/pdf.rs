use std::sync::Arc;

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

    pub fn with_standard(mut self, v: PdfStandard) -> Self {
        self.standards = Some(PdfStandards::new(&[v]).unwrap());
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
        let standards: PdfStandards = match self.standards.clone() {
            None => PdfStandards::default(),
            Some(standards) => standards,
        };

        typst_pdf::pdf(
            output.as_ref(),
            &PdfOptions {
                timestamp: self.ctime,
                standards,
                ..Default::default()
            },
        )
    }
}
