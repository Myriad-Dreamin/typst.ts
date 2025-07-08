//! reflexo-typst library.
//!
//! This library is used to compile Typst code into a document and export it
//! into various artifacts.
//! See <https://github.com/Myriad-Dreamin/typst.ts/tree/main/exporter> for
//! more information about the available exporters.

// The library consists of three parts:
// - `model`: low-level abstraction specific to the compiler, which defines:
//   - [`font::FontSlot`]: the way to load a font.
//   - [`vfs::AccessModel`]: how the compiler accesses a storage.
//   - [`package::PackageRegistry`]: how the compiler obtains data about a
//     package.
//
// - [`world`]: The world is the core part of the library, which maintains all
//   the data for typst compilation.
//   - [`vfs::Vfs`]: retrieving [`vfs::AccessModel`], provides a virtual file
//     system for the [`world::CompilerWorld`]
//   - [`world::CompilerWorld`]: retrieving [`world::CompilerFeat`], provides a
//     common implementation of [`::typst::World`].
//
// - `compile`: Convenient services over [`world::CompilerWorld`], which also
//   shows how to use the [`world::CompilerWorld`].
//   - [`CompileDriver`]: A driver for the compiler. Examples:
//     - Single thread (Sync): <https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/main.rs>
//     - Multiple thread (Async): <https://github.com/Enter-tainer/typst-preview-vscode/blob/main/src/main.rs>

// Core type system/concepts of typst-ts.
// #![warn(missing_docs)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_copy_implementations)]

pub mod config;
pub mod error;
pub mod query;
pub mod task;

#[cfg(feature = "hast")]
pub use reflexo_typst2hast::hast;

pub use concepts::*;
pub use error::{CompileReport, CompileReportMsg};

/// time things about compiler.
pub use reflexo::time;
pub use time::Time;
/// A common implementation of [`::typst::World`]
pub use tinymist_world as world;
/// A vfs implementation for compiler.
pub use tinymist_world::vfs;
/// font things about compiler.
pub use world::font;
/// package things about compiler.
pub use world::package;
/// Diff and parse the source code.
pub use world::parser;
pub use world::*;

pub use exporter::DynComputation;
pub use reflexo::typst_shim as compat;
pub use reflexo::*;
pub use reflexo_typst2vec as vector;
pub use reflexo_typst2vec::{debug_loc, hash};

#[cfg(feature = "ast")]
pub use exporter::ast::{dump_ast, AstExport, ExportAstTask};
#[cfg(feature = "svg")]
#[cfg(feature = "dynamic-layout")]
pub use exporter::dyn_svg::*;
#[cfg(feature = "html")]
pub use exporter::html::*;
#[cfg(feature = "svg")]
pub use exporter::svg::*;
pub use exporter::text::TextExport;
#[cfg(feature = "svg")]
pub use reflexo_vec2svg as svg;
pub use tinymist_task::compute::*;

#[cfg(feature = "system-watch")]
pub use compile::*;
pub use diag::DiagnosticFormat;
#[cfg(feature = "system-compile")]
pub use driver::*;
#[cfg(feature = "system-watch")]
pub use watch::*;

pub use ::typst::{Feature, Features};

#[cfg(feature = "system-compile")]
pub type DynSystemComputation = DynComputation<SystemCompilerFeat>;

#[cfg(feature = "system-watch")]
mod compile;
mod concepts;
#[cfg(feature = "system-compile")]
mod driver;
mod exporter;
mod utils;
#[cfg(feature = "system-watch")]
mod watch;

use ::typst::foundations::Content;
use ::typst::{
    diag::{At, SourceResult},
    syntax::Span,
};
use query::retrieve;

pub mod build_info {
    /// The version of the reflexo-typst crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}

pub mod diag {
    // todo: remove cfg feature here
    #[cfg(feature = "system-compile")]
    pub use tinymist_world::system::print_diagnostics;
    pub use tinymist_world::DiagnosticFormat;
}

pub trait CompilerExt<F: CompilerFeat> {
    fn world(&self) -> &CompilerWorld<F>;

    fn must_main_id(&self) -> TypstFileId {
        self.world().main()
    }

    /// With **the compilation state**, query the matches for the selector.
    fn query(&self, selector: String, document: &TypstDocument) -> SourceResult<Vec<Content>> {
        retrieve(&self.world(), &selector, document).at(Span::detached())
    }
}

impl<F: CompilerFeat> CompilerExt<F> for WorldComputeGraph<F> {
    fn world(&self) -> &CompilerWorld<F> {
        &self.snap.world
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_hash128() {
        assert_eq!(typst::utils::hash128(&0u32), reflexo::hash::hash128(&0u32));
    }
}
