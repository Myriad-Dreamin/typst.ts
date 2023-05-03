use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub trait DocumentExporter {
    /// Export the given document with given world.
    /// the writable world is hiden by trait itself.
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()>;
}

pub trait ArtifactExporter {
    /// Export the given artifact with given world.
    fn export(&self, world: &dyn World, output: Arc<crate::Artifact>) -> SourceResult<()>;
}

pub mod utils {
    use std::error::Error;
    use typst::{
        diag::{SourceError, SourceResult},
        World,
    };

    /// Convert the given error to a vector of source errors.
    pub fn map_err<E: Error>(world: &dyn World, e: E) -> Box<Vec<SourceError>> {
        Box::new(vec![SourceError::new(
            typst::syntax::Span::new(world.main().id(), 0),
            e.to_string(),
        )])
    }

    /// Export document to file system
    pub fn write_to_path<C: AsRef<[u8]>>(
        world: &dyn World,
        path: Option<std::path::PathBuf>,
        content: C,
    ) -> SourceResult<()> {
        path.map_or(Ok(()), |path| {
            std::fs::write(path, content).map_err(|e| map_err(world, e))
        })
    }
}
