pub mod artifact;
pub use artifact::Artifact;

pub mod artifact_ir;

pub mod config;

pub mod content;
pub use content::TextContent;

pub(crate) mod exporter;
pub use exporter::{utils as exporter_utils, ArtifactExporter, DocumentExporter};

pub mod font;
pub use font::{FontLoader, FontResolver, FontSlot};

pub(crate) mod hash;
pub use hash::typst_affinite_hash;

pub mod build_info {
    /// The version of the typst-ts-core crate.
    // todo: hard code it instead of using env!("CARGO_PKG_VERSION").
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}
