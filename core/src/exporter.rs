use typst::{diag::SourceResult, World};

pub trait DocExporter {
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()>;
}
