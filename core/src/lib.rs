// Core type system/concepts of typst-ts.
// #![warn(missing_docs)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_copy_implementations)]

mod concepts;
pub use concepts::*;

// Core data structures of typst-ts.
pub mod config;
pub mod error;

// Core mechanism of typst-ts.
pub(crate) mod exporter;

#[cfg(feature = "ast")]
pub use exporter::ast::{dump_ast, AstExporter};

pub use exporter::json::JsonExporter;

#[cfg(feature = "pdf")]
pub use exporter::pdf::PdfDocExporter;
#[cfg(feature = "pdf")]
pub use typst_pdf::pdf;

#[cfg(feature = "svg")]
pub use exporter::svg::*;
#[cfg(feature = "svg")]
pub use typst_ts_svg_exporter as svg;

pub use exporter::text::TextExporter;

pub use reflexo_typst2vec as vector;
pub use reflexo_typst2vec::debug_loc;
pub use reflexo_typst2vec::hash;

pub use exporter::{builtins as exporter_builtins, utils as exporter_utils};
pub use exporter::{
    DynExporter, DynGenericExporter, DynPolymorphicExporter, Exporter, GenericExporter,
    GenericTransformer, Transformer,
};
// pub use font::{FontLoader, FontResolver, FontSlot};
pub use reflexo::*;

pub mod build_info {
    /// The version of the typst-ts-core crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_hash128() {
        assert_eq!(typst::util::hash128(&0u32), reflexo::hash::hash128(&0u32));
    }
}
