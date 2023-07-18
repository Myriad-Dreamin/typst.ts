// Core type system/concepts of typst-ts.
pub(crate) mod concepts;
pub use concepts::*;
pub mod error;
pub use error::{ErrKind, Error};

pub mod hash;
pub use hash::typst_affinite_hash;
pub mod path;

// Core data structures of typst-ts.
pub mod annotation;
pub mod content;
pub use content::TextContent;
// todo: move me to compiler
pub mod cache;
pub mod config;

// Core mechanism of typst-ts.
pub(crate) mod exporter;
pub use exporter::{builtins as exporter_builtins, utils as exporter_utils};
pub use exporter::{mark_transformer_lambda, Exporter, Transformer};
pub mod font;
pub use font::{FontLoader, FontResolver, FontSlot};
pub mod package;

// Intermediate representation of typst-ts.
pub mod artifact;
pub use artifact::{core::ArtifactMeta, Artifact};
pub mod artifact_ir;
pub mod vector;

pub mod build_info {
    /// The version of the typst-ts-core crate.
    // todo: hard code it instead of using env!("CARGO_PKG_VERSION").
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}
