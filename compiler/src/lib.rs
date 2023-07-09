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

pub(crate) mod macros;

/// font things about compiler.
pub mod font;
/// package things about compiler.
pub mod package;
/// A vfs implementation for compiler.
pub mod vfs;
/// Workspace management for compiler.
pub mod workspace;
/// A common implementation of [`typst::World`]
pub mod world;

/// Convenient services over [`world::CompilerWorld`].
#[cfg(feature = "system")]
pub mod service;

/// Run the compiler in the system environment.
#[cfg(feature = "system")]
pub(crate) mod system;
#[cfg(feature = "system")]
pub use system::TypstSystemWorld;

/// Run the compiler in the browser environment.
#[cfg(feature = "browser-compile")]
pub(crate) mod browser;
#[cfg(feature = "browser-compile")]
pub use browser::TypstBrowserWorld;
