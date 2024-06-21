//! upstream of following files <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs>
//!   ::path_interner.rs -> path_interner.rs

/// Provides ProxyAccessModel that makes access to JavaScript objects for
/// browser compilation.
#[cfg(feature = "browser")]
pub mod browser;

/// Provides SystemAccessModel that makes access to the local file system for
/// system compilation.
#[cfg(feature = "system")]
pub mod system;

/// Provides dummy access model.
///
/// Note: we can still perform compilation with dummy access model, since
/// [`Vfs`] will make a overlay access model over the provided dummy access
/// model.
pub mod dummy;
/// Provides notify access model which retrieves file system events and changes
/// from some notify backend.
pub mod notify;
/// Provides overlay access model which allows to shadow the underlying access
/// model with memory contents.
pub mod overlay;
/// Provides trace access model which traces the underlying access model.
pub mod trace;
mod utils;

mod path_interner;

pub use typst::foundations::Bytes;
pub use typst::syntax::FileId as TypstFileId;

pub use reflexo::time::Time;
pub use reflexo::ImmutPath;

pub(crate) use path_interner::PathInterner;

use core::fmt;
use std::{collections::HashMap, hash::Hash, path::Path, sync::Arc};

use parking_lot::{Mutex, RwLock};
use reflexo::path::PathClean;
use typst::diag::{FileError, FileResult};

use self::{
    notify::{FilesystemEvent, NotifyAccessModel},
    overlay::OverlayAccessModel,
};

/// Handle to a file in [`Vfs`]
///
/// Most functions in typst-ts use this when they need to refer to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a new type of `u32`
impl nohash_hasher::IsEnabled for FileId {}

/// A trait for accessing underlying file system.
///
/// This trait is simplified by [`Vfs`] and requires a minimal method set for
/// typst compilation.
pub trait AccessModel {
    /// Clear the cache of the access model.
    ///
    /// This is called when the vfs is reset. See [`Vfs`]'s reset method for
    /// more information.
    fn clear(&mut self) {}

    /// Return a mtime corresponding to the path.
    ///
    /// Note: vfs won't touch the file entry if mtime is same between vfs reset
    /// lifecycles for performance design.
    fn mtime(&self, src: &Path) -> FileResult<Time>;

    /// Return whether a path is corresponding to a file.
    fn is_file(&self, src: &Path) -> FileResult<bool>;

    /// Return the real path before creating a vfs file entry.
    ///
    /// Note: vfs will fetch the file entry once if multiple paths shares a same
    /// real path.
    fn real_path(&self, src: &Path) -> FileResult<ImmutPath>;

    /// Return the content of a file entry.
    fn content(&self, src: &Path) -> FileResult<Bytes>;
}

/// we add notify access model here since notify access model doesn't introduce
/// overheads by our observation
type VfsAccessModel<M> = OverlayAccessModel<NotifyAccessModel<M>>;

pub trait FsProvider {
    fn file_path(&self, src: FileId) -> ImmutPath;

    fn mtime(&self, src: FileId) -> FileResult<Time>;

    fn read(&self, src: FileId) -> FileResult<Bytes>;

    fn is_file(&self, src: FileId) -> FileResult<bool>;
}

/// Create a new `Vfs` harnessing over the given `access_model` specific for
/// [`crate::world::CompilerWorld`]. With vfs, we can minimize the
/// implementation overhead for [`AccessModel`] trait.
pub struct Vfs<M: AccessModel + Sized> {
    /// The number of lifecycles since the creation of the `Vfs`.
    ///
    /// Note: The lifetime counter is incremented on resetting vfs.
    lifetime_cnt: u64,

    // access_model: TraceAccessModel<VfsAccessModel<M>>,
    /// The wrapped access model.
    access_model: VfsAccessModel<M>,
    /// The path interner for canonical paths.
    path_interner: Mutex<PathInterner<ImmutPath, u64>>,

    /// Map from path to slot index.
    ///
    /// Note: we use a owned [`FileId`] here, which is resultant from
    /// [`PathInterner`]
    path2slot: RwLock<HashMap<ImmutPath, FileId>>,
}

impl<M: AccessModel + Sized> fmt::Debug for Vfs<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs")
            .field("lifetime_cnt", &self.lifetime_cnt)
            .field("path2slot", &self.path2slot)
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
        let access_model = NotifyAccessModel::new(access_model);
        let access_model = OverlayAccessModel::new(access_model);

        // If you want to trace the access model, uncomment the following line
        // let access_model = TraceAccessModel::new(access_model);

        Self {
            lifetime_cnt: 0,
            access_model,
            path_interner: Mutex::new(PathInterner::default()),
            path2slot: RwLock::new(HashMap::new()),
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
        // todo: clean path interner.
        // self.lifetime_cnt += 1;
        // let new_lifetime_cnt = self.lifetime_cnt;

        // self.path2slot.get_mut().clear();
        // self.path_interner
        //     .get_mut()
        //     .retain(|_, lifetime| new_lifetime_cnt - *lifetime <= 30);

        self.access_model.clear();
    }

    /// Reset the shadowing files in [`OverlayAccessModel`].
    ///
    /// Note: This function is independent from [`Vfs::reset`].
    pub fn reset_shadow(&mut self) {
        self.access_model.clear_shadow();
    }

    /// Get paths to all the shadowing files in [`OverlayAccessModel`].
    pub fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.access_model.file_paths()
    }

    /// Add a shadowing file to the [`OverlayAccessModel`].
    pub fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.access_model.add_file(path.into(), content);

        Ok(())
    }

    /// Remove a shadowing file from the [`OverlayAccessModel`].
    pub fn remove_shadow(&self, path: &Path) {
        self.access_model.remove_file(path);
    }

    /// Let the vfs notify the access model with a filesystem event.
    ///
    /// See [`NotifyAccessModel`] for more information.
    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.access_model.inner.notify(event);
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        0
    }

    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &Path) -> FileId {
        let quick_id = self.path2slot.read().get(path).copied();
        if let Some(id) = quick_id {
            return id;
        }

        let path: ImmutPath = path.clean().as_path().into();

        let mut path_interner = self.path_interner.lock();
        let id = path_interner.intern(path.clone(), self.lifetime_cnt).0;

        let mut path2slot = self.path2slot.write();
        path2slot.insert(path.clone(), id);

        id
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> ImmutPath {
        let path_interner = self.path_interner.lock();
        path_interner.lookup(file_id).clone()
    }

    /// Read a file.
    pub fn read(&self, path: &Path) -> FileResult<Bytes> {
        if self.access_model.is_file(path)? {
            self.access_model.content(path)
        } else {
            Err(FileError::IsDirectory)
        }
    }
}

impl<M: AccessModel> FsProvider for Vfs<M> {
    fn file_path(&self, src: FileId) -> ImmutPath {
        self.file_path(src)
    }

    fn mtime(&self, src: FileId) -> FileResult<Time> {
        self.access_model.inner.mtime(&self.file_path(src))
    }

    fn read(&self, src: FileId) -> FileResult<Bytes> {
        self.access_model.inner.content(&self.file_path(src))
    }

    fn is_file(&self, src: FileId) -> FileResult<bool> {
        self.access_model.inner.is_file(&self.file_path(src))
    }
}

#[cfg(test)]
mod tests {
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    #[test]
    fn test_vfs_send_sync() {
        is_send::<super::Vfs<super::dummy::DummyAccessModel>>();
        is_sync::<super::Vfs<super::dummy::DummyAccessModel>>();
    }
}
