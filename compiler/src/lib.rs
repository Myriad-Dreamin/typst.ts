//! Typst.ts compiler library.
//!
//! This library is used to compile Typst code into a document and export it
//! into various artifacts.
//! See <https://github.com/Myriad-Dreamin/typst.ts/tree/main/exporter> for
//! more information about the available exporters.
//!
//! The library consists of three parts:
//! - `model`: low-level abstraction specific to the compiler, which defines:
//!   - [`font::FontSlot`]: the way to load a font.
//!   - [`vfs::AccessModel`]: how the compiler accesses a storage.
//!   - [`package::Registry`]: how the compiler obtains data about a package.
//!
//! - [`world`]: The world is the core part of the library, which maintains all
//!   the data for typst compilation.
//!   - [`vfs::Vfs`]: retrieving [`vfs::AccessModel`], provides a virtual file
//!     system for the [`world::CompilerWorld`]
//!   - [`world::CompilerWorld`]: retrieving [`world::CompilerFeat`], provides a
//!     common implementation of [`typst::World`].
//!
//! - [`service`]: Convenient services over [`world::CompilerWorld`], which also
//!   shows how to use the [`world::CompilerWorld`].
//!   - [`service::CompileDriver`]: A driver for the compiler. Examples:
//!     - Single thread (Sync): <https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/main.rs>
//!     - Multiple thread (Async): <https://github.com/Enter-tainer/typst-preview-vscode/blob/main/src/main.rs>

/// font things about compiler.
pub use world::font;

/// package things about compiler.
pub use world::package;
/// time things about compiler.
pub mod time;
/// A vfs implementation for compiler.
pub use reflexo_vfs as vfs;
/// A common implementation of [`typst::World`]
pub use reflexo_world as world;
pub use world::*;

pub mod eval;
/// Diff and parse the source code.
pub use world::parser;
mod utils;

/// Convenient services over [`world::CompilerWorld`].
pub mod service;

pub use time::Time;
