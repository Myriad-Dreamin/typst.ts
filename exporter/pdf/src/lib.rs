pub use typst::export::pdf;
use typst_ts_core::DocumentExporter;

use typst::{diag::SourceResult, World};
pub(crate) use typst_ts_core::exporter_utils::*;

pub struct PdfDocExporter {
    path: Option<std::path::PathBuf>,
}

impl PdfDocExporter {
    pub fn new_path(path: std::path::PathBuf) -> Self {
        Self { path: Some(path) }
    }
}

impl DocumentExporter for PdfDocExporter {
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()> {
        let buffer = typst::export::pdf(&output);
        write_to_path(world, self.path.clone(), buffer)
    }
}
