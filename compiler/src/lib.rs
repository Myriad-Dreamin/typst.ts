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
/// time things about compiler.
pub mod time;
/// A vfs implementation for compiler.
pub mod vfs;
/// Workspace management for compiler.
pub mod workspace;
/// A common implementation of [`typst::World`]
pub mod world;

/// Diff and parse the source code.
mod parser;

/// Convenient services over [`world::CompilerWorld`].
pub mod service;

/// Run the compiler in the system environment.
#[cfg(feature = "system-compile")]
pub(crate) mod system;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

#[cfg(feature = "system-compile")]
pub use system::TypstSystemWorld;

/// Run the compiler in the browser environment.
#[cfg(feature = "browser-compile")]
pub(crate) mod browser;
#[cfg(feature = "browser-compile")]
pub use browser::TypstBrowserWorld;
use typst::{
    diag::{At, FileResult, SourceResult},
    syntax::Span,
};
use typst_ts_core::{Bytes, ImmutPath, TypstFileId};
use vfs::notify::FilesystemEvent;

/// Latest version of the shadow api, which is in beta.
pub trait ShadowApi {
    fn _shadow_map_id(&self, _file_id: TypstFileId) -> FileResult<PathBuf> {
        unimplemented!()
    }

    /// Get the shadow files.
    fn shadow_paths(&self) -> Vec<Arc<Path>>;

    /// Reset the shadow files.
    fn reset_shadow(&mut self) {
        for path in self.shadow_paths() {
            self.unmap_shadow(&path).unwrap();
        }
    }

    /// Add a shadow file to the driver.
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()>;

    /// Add a shadow file to the driver.
    fn unmap_shadow(&self, path: &Path) -> FileResult<()>;

    /// Wrap the driver with a given shadow file and run the inner function.
    fn with_shadow_file<T>(
        &mut self,
        file_path: &Path,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T> {
        self.map_shadow(file_path, content).at(Span::detached())?;
        let res: Result<T, Box<Vec<typst::diag::SourceDiagnostic>>> = f(self);
        self.unmap_shadow(file_path).at(Span::detached())?;
        res
    }

    /// Add a shadow file to the driver by file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn map_shadow_by_id(&self, file_id: TypstFileId, content: Bytes) -> FileResult<()> {
        let file_path = self._shadow_map_id(file_id)?;
        self.map_shadow(&file_path, content)
    }

    /// Add a shadow file to the driver by file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn unmap_shadow_by_id(&self, file_id: TypstFileId) -> FileResult<()> {
        let file_path = self._shadow_map_id(file_id)?;
        self.unmap_shadow(&file_path)
    }

    /// Wrap the driver with a given shadow file and run the inner function by
    /// file id.
    /// Note: to enable this function, `ShadowApi` must implement
    /// `_shadow_map_id`.
    fn with_shadow_file_by_id<T>(
        &mut self,
        file_id: TypstFileId,
        content: Bytes,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> SourceResult<T> {
        let file_path = self._shadow_map_id(file_id).at(Span::detached())?;
        self.with_shadow_file(&file_path, content, f)
    }
}

/// Latest version of the notify api, which is in beta.
pub trait NotifyApi {
    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, instant::SystemTime));

    fn notify_fs_event(&mut self, event: FilesystemEvent);
}
