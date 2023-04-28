pub use typst::export::pdf;
use typst_ts_core::DocExporter;

use std::error::Error;

use typst::{diag::SourceResult, World};

/// Convert the given error to a vector of source errors.
fn map_err<E: Error>(world: &dyn World, e: E) -> Box<Vec<typst::diag::SourceError>> {
    Box::new(vec![typst::diag::SourceError::new(
        typst::syntax::Span::new(world.main().id(), 0),
        e.to_string(),
    )])
}

/// export document to file system
fn write_to_path<C: AsRef<[u8]>>(
    world: &dyn World,
    path: Option<std::path::PathBuf>,
    content: C,
) -> SourceResult<()> {
    path.map_or(Ok(()), |path| {
        std::fs::write(path, content).map_err(|e| map_err(world, e))
    })
}
pub struct PdfDocExporter {
    path: Option<std::path::PathBuf>,
}

impl PdfDocExporter {
    pub fn new_path(path: std::path::PathBuf) -> Self {
        Self { path: Some(path) }
    }
}

impl DocExporter for PdfDocExporter {
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()> {
        let buffer = typst::export::pdf(&output);
        write_to_path(world, self.path.clone(), buffer)
    }
}
