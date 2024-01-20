// Core type system/concepts of typst-ts.
// #![warn(missing_docs)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_copy_implementations)]

pub use reflexo::*;

pub use hash::typst_affinite_hash;

mod concepts2 {

    /// This is an implementation for `Write + !AsRef<AnyBytes>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AsWritable;

    /// This is an implementation for `Vec<u8>`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AsOwnedBytes;

    /// This is an implementation for `String`.
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AsOwnedString;
}
pub use concepts2::*;

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
pub use exporter::{
    DynExporter, DynGenericExporter, DynPolymorphicExporter, Exporter, GenericExporter, Transformer,
};
pub mod font;
pub use font::{FontLoader, FontResolver, FontSlot};
pub mod package;

// Intermediate representation of typst-ts.
pub mod vector;

pub mod build_info {
    /// The version of the typst-ts-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}
