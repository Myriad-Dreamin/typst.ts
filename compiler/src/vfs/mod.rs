//! upstream of following files <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs>
//!   ::path_interner.rs -> path_interner.rs

/// Provides ProxyAccessModel that makes access to JavaScript objects for
/// browser compilation.
#[cfg(feature = "browser-compile")]
pub mod browser;

/// Provides SystemAccessModel that makes access to the local file system for
/// system compilation.
#[cfg(feature = "system-compile")]
pub mod system;

/// Provides general cache to file access.
pub mod cached;
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

mod path_interner;

pub(crate) use path_interner::PathInterner;

use core::fmt;
use std::{collections::HashMap, ffi::OsStr, hash::Hash, path::Path, sync::Arc};

use append_only_vec::AppendOnlyVec;
use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use typst::{
    diag::{FileError, FileResult},
    syntax::Source,
};

use typst_ts_core::{path::PathClean, Bytes, ImmutPath, QueryRef, TypstFileId};

use crate::{parser::reparse, Time};

use self::{
    cached::CachedAccessModel,
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
    /// The real path type for the underlying file system.
    /// This type is used for canonicalizing paths.
    type RealPath: Hash + Eq + PartialEq + for<'a> From<&'a Path>;

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
    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath>;

    /// Return the content of a file entry.
    fn content(&self, src: &Path) -> FileResult<Bytes>;
}

type FileQuery<T> = QueryRef<T, FileError>;

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Debug)]
pub struct PathSlot {
    idx: FileId,
    sampled_path: once_cell::sync::OnceCell<ImmutPath>,
    mtime: FileQuery<Time>,
    source: FileQuery<Source>,
    buffer: FileQuery<Bytes>,
}

impl PathSlot {
    /// Create a new slot with a given local file id from [`PathInterner`].
    fn new(idx: FileId) -> Self {
        PathSlot {
            idx,
            sampled_path: once_cell::sync::OnceCell::new(),
            mtime: FileQuery::default(),
            source: FileQuery::default(),
            buffer: FileQuery::default(),
        }
    }
}

/// we add notify access model here since notify access model doesn't introduce
/// overheads by our observation
type VfsAccessModel<M> = CachedAccessModel<OverlayAccessModel<NotifyAccessModel<M>>, Source>;

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
    path_interner: Mutex<PathInterner<<M as AccessModel>::RealPath, u64>>,

    /// Map from path to slot index.
    ///
    /// Note: we use a owned [`FileId`] here, which is resultant from
    /// [`PathInterner`]
    path2slot: RwLock<HashMap<Arc<OsStr>, FileId>>,
    /// Map from typst global file id to a local file id.
    src2file_id: RwLock<HashMap<TypstFileId, FileId>>,
    /// The slots for all the files during a single lifecycle.
    pub slots: AppendOnlyVec<PathSlot>,
    /// Whether to reparse the file when it is changed.
    /// Default to `true`.
    pub do_reparse: bool,
}

impl<M: AccessModel + Sized> fmt::Debug for Vfs<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Vfs")
            .field("lifetime_cnt", &self.lifetime_cnt)
            .field("path2slot", &self.path2slot)
            .field("src2file_id", &self.src2file_id)
            .field("slots", &self.slots)
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
        let access_model = NotifyAccessModel::new(access_model);
        let access_model = OverlayAccessModel::new(access_model);
        let access_model = CachedAccessModel::new(access_model);

        // If you want to trace the access model, uncomment the following line
        // let access_model = TraceAccessModel::new(access_model);

        Self {
            lifetime_cnt: 0,
            access_model,
            path_interner: Mutex::new(PathInterner::default()),
            slots: AppendOnlyVec::new(),
            src2file_id: RwLock::new(HashMap::new()),
            path2slot: RwLock::new(HashMap::new()),
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
        self.lifetime_cnt += 1;
        let new_lifetime_cnt = self.lifetime_cnt;

        self.slots = AppendOnlyVec::new();
        self.path2slot.get_mut().clear();
        self.src2file_id.get_mut().clear();

        self.path_interner
            .get_mut()
            .retain(|_, lifetime| new_lifetime_cnt - *lifetime <= 30);

        self.access_model.clear();
    }

    /// Reset the shadowing files in [`OverlayAccessModel`].
    ///
    /// Note: This function is independent from [`Vfs::reset`].
    pub fn reset_shadow(&mut self) {
        self.access_model.inner().clear_shadow();
    }

    /// Get paths to all the shadowing files in [`OverlayAccessModel`].
    pub fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.access_model.inner().file_paths()
    }

    /// Add a shadowing file to the [`OverlayAccessModel`].
    pub fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.access_model.inner().add_file(path.into(), content);

        Ok(())
    }

    /// Remove a shadowing file from the [`OverlayAccessModel`].
    pub fn remove_shadow(&self, path: &Path) {
        self.access_model.inner().remove_file(path);
    }

    /// Let the vfs notify the access model with a filesystem event.
    ///
    /// See [`NotifyAccessModel`] for more information.
    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.access_model.inner_mut().inner_mut().notify(event);
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
        let mut w = self.slots.len() * core::mem::size_of::<PathSlot>();
        w += self.path2slot.read().capacity() * 256;
        w += self.src2file_id.read().capacity() * 16;
        w += self
            .slots
            .iter()
            .map(|slot| {
                slot.source
                    .get_uninitialized()
                    .and_then(|e| e.as_ref().ok())
                    .map_or(16, |e| e.text().len() * 8)
                    + slot
                        .buffer
                        .get_uninitialized()
                        .and_then(|e| e.as_ref().ok())
                        .map_or(16, |e| e.len())
            })
            .sum::<usize>();

        w
    }

    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        let path = path.clean();
        self.path2slot.read().get(path.as_os_str()).copied()
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> &Path {
        self.slots[file_id.0 as usize].sampled_path.get().unwrap()
    }

    /// Get all the files that are currently in the VFS.
    ///
    /// This is typically corresponds to the file dependencies of a single
    /// compilation.
    ///
    /// When you don't reset the vfs for each compilation, this function will
    /// still return remaining files from the previous compilation.
    pub fn iter_dependencies(&self) -> impl Iterator<Item = (&ImmutPath, Time)> {
        self.slots.iter().map(|slot| {
            let dep_path = slot.sampled_path.get().unwrap();
            let dep_mtime = slot
                .mtime
                .compute(|| Err(other_reason("vfs: uninitialized")))
                .unwrap();

            (dep_path, *dep_mtime)
        })
    }

    /// Get all the files that are currently in the VFS. This function is
    /// similar to [`Vfs::iter_dependencies`], but it is for trait objects.
    pub fn iter_dependencies_dyn<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, Time)) {
        for slot in self.slots.iter() {
            let Some(dep_path) = slot.sampled_path.get() else {
                continue;
            };
            let Ok(dep_mtime) = slot
                .mtime
                .compute(|| Err(other_reason("vfs: uninitialized")))
            else {
                continue;
            };

            f(dep_path, *dep_mtime)
        }
    }

    /// Read a file.
    fn read(&self, path: &Path) -> FileResult<Bytes> {
        if self.access_model.is_file(path)? {
            self.access_model.content(path)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    /// Get file content by path.
    pub fn file(&self, path: &Path) -> FileResult<Bytes> {
        let slot = self.slot(path)?;

        let buffer = slot.buffer.compute(|| self.read(path))?;
        Ok(buffer.clone())
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
            if self.access_model.is_file(path)? {
                Ok(self
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
        // If we cannot get the real path, keep the origin path syntactically.
        let real_path = self
            .access_model
            .real_path(origin_path)
            .unwrap_or_else(|_| origin_path.into());

        let mut path_interner = self.path_interner.lock();
        let (file_id, _) = path_interner.intern(real_path, self.lifetime_cnt);
        let idx = file_id.0 as usize;
        for i in self.slots.len()..idx + 1 {
            self.slots.push(PathSlot::new(FileId(i as u32)));
        }

        let slot = &self.slots[idx];
        slot.sampled_path.get_or_init(|| origin_path.into());
        Ok(&self.slots[idx])
    }

    /// Insert a new slot into the vfs.
    fn slot(&self, origin_path: &Path) -> FileResult<&PathSlot> {
        // fast path for already inserted paths
        let path2slot = self.path2slot.upgradable_read();
        if let Some(slot) = path2slot.get(origin_path.as_os_str()) {
            return Ok(&self.slots[slot.0 as usize]);
        }

        // get slot for the path
        let slot = self.get_real_slot(origin_path)?;

        // insert the slot into the path2slot map
        // note: path aliases will share the same slot
        let mut path2slot = RwLockUpgradableReadGuard::upgrade(path2slot);
        let inserted = path2slot.insert(origin_path.as_os_str().into(), slot.idx);
        assert!(inserted.is_none(), "slot already inserted");

        let normalized = origin_path.clean();
        if path2slot.get(normalized.as_os_str()).is_none() {
            let inserted = path2slot.insert(normalized.as_os_str().into(), slot.idx);
            assert!(inserted.is_none(), "slot already inserted");
        }

        // prefetch a early mtime
        slot.mtime
            .compute(|| self.access_model.mtime(origin_path))?;

        Ok(slot)
    }

    /// Get source content by path with a read content implementation.
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
            .cloned()
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
