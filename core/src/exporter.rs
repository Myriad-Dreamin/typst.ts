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

pub struct DocToArtifactExporter {
    artifact_exporters: Vec<Box<dyn ArtifactExporter>>,
}

impl DocToArtifactExporter {
    pub fn new(artifact_exporters: Vec<Box<dyn ArtifactExporter>>) -> Self {
        Self { artifact_exporters }
    }
}

impl DocumentExporter for DocToArtifactExporter {
    fn export(&self, world: &dyn World, output: &typst::doc::Document) -> SourceResult<()> {
        let mut errors = Vec::new();

        let artifact = Arc::new(crate::Artifact::from(output));
        for f in &self.artifact_exporters {
            utils::collect_err(&mut errors, f.export(world, artifact.clone()))
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(Box::new(errors))
        }
    }
}

pub mod utils {
    use std::error::Error;
    use typst::{
        diag::{SourceError, SourceResult},
        World,
    };

    pub fn collect_err(errors: &mut Vec<SourceError>, res: SourceResult<()>) {
        if let Err(errs) = res {
            let mut errs = *errs;
            errors.append(&mut errs);
        }
    }

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
