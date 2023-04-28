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

pub(crate) mod macros;

#[cfg(feature = "json")]
pub(crate) mod json;
#[cfg(feature = "json")]
pub use json::JsonArtifactExporter;

#[cfg(feature = "rmp")]
pub(crate) mod rmp;
#[cfg(feature = "rmp")]
pub use rmp::RmpArtifactExporter;
