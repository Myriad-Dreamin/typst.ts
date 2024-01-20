//! upstream of following files <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs>
//!   ::path_interner.rs -> path_interner.rs

pub use reflexo_vfs::*;

use core::fmt;
use std::{collections::HashMap, ffi::OsStr, hash::Hash, path::Path, sync::Arc};

use append_only_vec::AppendOnlyVec;
use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use typst::syntax::Source;

use crate::TypstFileId;
use reflexo::{
    error::{FileError, FileResult},
    path::PathClean,
    Bytes, ImmutPath, QueryRef,
};

use crate::{parser::reparse, Time};

use self::{
    cached::CachedAccessModel,
    notify::{FilesystemEvent, NotifyAccessModel},
    overlay::OverlayAccessModel,
};

type FileQuery<T> = QueryRef<T, FileError>;

/// Holds canonical data for all paths pointing to the same entity.
pub type PathSlot = reflexo_vfs::PathSlot<Source>;

/// we add notify access model here since notify access model doesn't introduce
/// overheads by our observation
type VfsAccessModel<M> = CachedAccessModel<OverlayAccessModel<NotifyAccessModel<M>>, Source>;

/// Create a new `Vfs` harnessing over the given `access_model` specific for
/// [`crate::world::CompilerWorld`]. With vfs, we can minimize the
/// implementation overhead for [`AccessModel`] trait.
pub struct Vfs<M: AccessModel + Sized> {
    /// The inner vfs
    inner: reflexo_vfs::Vfs<Source, M>,

    /// Map from typst global file id to a local file id.
    src2file_id: RwLock<HashMap<TypstFileId, FileId>>,

    /// Whether to reparse the file when it is changed.
    /// Default to `true`.
    pub do_reparse: bool,
}

impl<M: AccessModel + Sized> fmt::Debug for Vfs<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs")
            .field("inner", &self.inner)
            .field("src2file_id", &self.src2file_id)
            .field("do_reparse", &self.do_reparse)
            .finish()
    }
}

impl<M: AccessModel + Sized> Vfs<M> {
    /// Create a new `Vfs` with a given `access_model`.
    ///
    /// Retrieving an [`AccessModel`], it will further wrap the access model
    /// with [`CachedAccessModel`], [`OverlayAccessModel`], and
    /// [`NotifyAccessModel`]. This means that you don't need to implement:
    /// + cache: caches underlying access result for a single vfs lifecycle,
    ///   typically also corresponds to a single compilation.
    /// + overlay: allowing to shadow the underlying access model with memory
    ///   contents, which is useful for a limited execution environment and
    ///   instrumenting or overriding source files or packages.
    /// + notify: regards problems of synchronizing with the file system when
    ///   the vfs is watching the file system.
    ///
    /// See [`AccessModel`] for more information.
    pub fn new(access_model: M) -> Self {
        let inner = reflexo_vfs::Vfs::new(access_model);

        Self {
            inner,
            src2file_id: RwLock::new(HashMap::new()),
            do_reparse: true,
        }
    }

    /// Reset the source file and path references.
    ///
    /// It performs a rolling reset, with discard some cache file entry when it
    /// is unused in recent 30 lifecycles.
    ///
    /// Note: The lifetime counter is incremented every time this function is
    /// called.
    pub fn reset(&mut self) {
        self.inner.reset();

        self.src2file_id.get_mut().clear();
    }

    /// Reset the shadowing files in [`OverlayAccessModel`].
    ///
    /// Note: This function is independent from [`Vfs::reset`].
    pub fn reset_shadow(&mut self) {
        self.inner.reset_shadow();
    }

    /// Get paths to all the shadowing files in [`OverlayAccessModel`].
    pub fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.inner.shadow_paths()
    }

    /// Add a shadowing file to the [`OverlayAccessModel`].
    pub fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.inner.map_shadow(path, content)
    }

    /// Remove a shadowing file from the [`OverlayAccessModel`].
    pub fn remove_shadow(&self, path: &Path) {
        self.inner.remove_shadow(path)
    }

    /// Let the vfs notify the access model with a filesystem event.
    ///
    /// See [`NotifyAccessModel`] for more information.
    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.inner.notify_fs_event(event)
    }

    /// Set the `do_reparse` flag that indicates whether to reparsing the file
    /// instead of creating a new [`Source`] when the file is changed.
    /// Default to `true`.
    ///
    /// You usually want to set this flag to `true` for better performance.
    /// However, one could disable this flag for debugging purpose.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.do_reparse = do_reparse;
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        self.inner.memory_usage()
    }

    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        self.inner.file_id(path)
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> &Path {
        self.inner.file_path(file_id)
    }

    /// Get all the files that are currently in the VFS.
    ///
    /// This is typically corresponds to the file dependencies of a single
    /// compilation.
    ///
    /// When you don't reset the vfs for each compilation, this function will
    /// still return remaining files from the previous compilation.
    pub fn iter_dependencies(&self) -> impl Iterator<Item = (&ImmutPath, Time)> {
        self.inner.iter_dependencies()
    }

    /// Get all the files that are currently in the VFS. This function is
    /// similar to [`Vfs::iter_dependencies`], but it is for trait objects.
    pub fn iter_dependencies_dyn<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, Time)) {
        self.inner.iter_dependencies_dyn(f)
    }

    /// Read a file.
    fn read(&self, path: &Path) -> FileResult<Bytes> {
        self.inner.read(path)
    }

    /// Get file content by path.
    pub fn file(&self, path: &Path) -> FileResult<Bytes> {
        self.inner.file(path)
    }

    /// Get source content by path and assign the source with a given typst
    /// global file id.
    ///
    /// See `Vfs::resolve_with_f` for more information.
    pub fn resolve(&self, path: &Path, source_id: TypstFileId) -> FileResult<Source> {
        self.resolve_with_f(path, source_id, || {
            // Return a new source if we don't have a reparse feature
            if !self.do_reparse {
                let content = self.read(path)?;
                let content = from_utf8_or_bom(&content)?.to_owned();
                let res = Ok(Source::new(source_id, content));

                return res;
            }

            // otherwise reparse the source
            if self.inner.access_model.is_file(path)? {
                Ok(self
                    .inner
                    .access_model
                    .read_all_diff(path, |x, y| reparse(source_id, x, y))?)
            } else {
                Err(FileError::IsDirectory)
            }
        })
    }

    /// Get or insert a slot for a path. All paths pointing to the same entity
    /// will share the same slot.
    ///
    /// - If `path` does not exists in the `Vfs`, allocate a new id for it,
    ///   associated with a deleted file;
    /// - Else, returns `path`'s id.
    ///
    /// Does not record a change.
    fn get_real_slot(&self, origin_path: &Path) -> FileResult<&PathSlot> {
        self.inner.get_real_slot(origin_path)
    }

    /// Insert a new slot into the vfs.
    fn slot(&self, origin_path: &Path) -> FileResult<&PathSlot> {
        self.inner.slot(origin_path)
    }

    /// Get source content by path with a read content implmentation.
    ///
    /// Note: This function will also do eager check that whether the path
    /// exists in the underlying access model. So the read content function
    /// won't be triggered if the path doesn't exist.
    fn resolve_with_f<ReadContent: FnOnce() -> FileResult<Source>>(
        &self,
        path: &Path,
        source_id: TypstFileId,
        read: ReadContent,
    ) -> FileResult<Source> {
        let slot = self.slot(path)?;

        slot.source
            .compute(|| {
                self.src2file_id.write().insert(source_id, slot.idx);
                read()
            })
            .map(|e| e.clone())
    }
}

/// Convert a byte slice to a string, removing UTF-8 BOM if present.
fn from_utf8_or_bom(buf: &[u8]) -> FileResult<&str> {
    Ok(std::str::from_utf8(if buf.starts_with(b"\xef\xbb\xbf") {
        // remove UTF-8 BOM
        &buf[3..]
    } else {
        // Assume UTF-8
        buf
    })?)
}

/// Create a [`FileError`] with a given error message.
fn other_reason(err: &str) -> FileError {
    FileError::Other(Some(err.into()))
}
