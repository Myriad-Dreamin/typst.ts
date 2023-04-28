use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub trait DocExporter {
    /// Export the given document with given world.
    /// the writable world is hiden by trait itself.
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()>;
}

pub trait ArtifactExporter {
    /// Export the given artifact with given world.
    fn export(&self, world: &dyn World, output: Arc<crate::Artifact>) -> SourceResult<()>;
}
