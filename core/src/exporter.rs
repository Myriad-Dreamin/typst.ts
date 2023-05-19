use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub(crate) type DocumentRef = Arc<typst::doc::Document>;
pub(crate) type ArtifactRef = Arc<crate::Artifact>;

pub(crate) type DocumentExporter = Box<dyn Exporter<typst::doc::Document>>;
pub(crate) type ArtifactExporter = Box<dyn Exporter<crate::Artifact>>;

pub trait Exporter<Input, Output = ()> {
    /// Export the given input with given world.
    /// the writable world is hiden by trait itself.
    fn export(&self, world: &dyn World, output: Arc<Input>) -> SourceResult<Output>;
}

pub mod builtins {
    use super::{utils, ArtifactExporter, ArtifactRef, DocumentExporter, DocumentRef, Exporter};
    use typst::{diag::SourceResult, World};

    pub struct GroupDocumentExporter {
        document_exporters: Vec<Box<dyn Exporter<typst::doc::Document>>>,
    }

    impl GroupDocumentExporter {
        pub fn new(document_exporters: Vec<DocumentExporter>) -> Self {
            Self { document_exporters }
        }
    }

    impl Exporter<typst::doc::Document> for GroupDocumentExporter {
        fn export(&self, world: &dyn World, output: DocumentRef) -> SourceResult<()> {
            let mut errors = Vec::new();

            for f in &self.document_exporters {
                utils::collect_err(&mut errors, f.export(world, output.clone()))
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(Box::new(errors))
            }
        }
    }

    pub struct DocToArtifactExporter {
        artifact_exporters: Vec<ArtifactExporter>,
    }

    impl DocToArtifactExporter {
        pub fn new(artifact_exporters: Vec<ArtifactExporter>) -> Self {
            Self { artifact_exporters }
        }
    }

    impl Exporter<typst::doc::Document> for DocToArtifactExporter {
        fn export(&self, world: &dyn World, output: DocumentRef) -> SourceResult<()> {
            let mut errors = Vec::new();

            let artifact = ArtifactRef::new(output.as_ref().into());
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

    pub struct LambdaDocumentExporter<F> {
        f: F,
    }

    impl<F> LambdaDocumentExporter<F>
    where
        F: Fn(&dyn World, DocumentRef) -> SourceResult<()>,
    {
        pub fn new(f: F) -> Self {
            Self { f }
        }
    }

    impl<F> Exporter<typst::doc::Document> for LambdaDocumentExporter<F>
    where
        F: Fn(&dyn World, DocumentRef) -> SourceResult<()>,
    {
        fn export(&self, world: &dyn World, output: DocumentRef) -> SourceResult<()> {
            (self.f)(world, output)
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
