//! upstream of following files <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs>
//!   ::path_interner.rs -> path_interner.rs
//!   ::paths.rs -> abs_path.rs
//!   ::anchored_path.rs -> path_anchored.rs
//!   ::vfs_path.rs -> path_vfs.rs

#[cfg(feature = "browser-compile")]
pub mod browser;

#[cfg(feature = "system-compile")]
pub mod system;

pub mod cached;
pub mod dummy;
pub mod notify;
pub mod overlay;
pub mod trace;

mod path_abs;
mod path_anchored;
mod path_interner;
mod path_vfs;

pub(crate) use path_interner::PathInterner;
pub use {
    path_abs::{AbsPath, AbsPathBuf},
    path_anchored::{AnchoredPath, AnchoredPathBuf},
    path_vfs::VfsPath,
};

pub(crate) mod writable;
pub use writable::Vfs as MemVfs;
pub use writable::{ChangeKind, ChangedFile};

use std::{
    collections::HashMap,
    ffi::OsStr,
    hash::Hash,
    path::{Path, PathBuf},
    sync::Arc,
};

use append_only_vec::AppendOnlyVec;
use parking_lot::{Mutex, RwLock, RwLockUpgradableReadGuard};
use typst::{
    diag::{FileError, FileResult},
    syntax::Source,
};

use typst_ts_core::{path::PathClean, Bytes, QueryRef, TypstFileId};

use crate::{parser::reparse, time::SystemTime};

use self::{
    cached::CachedAccessModel,
    notify::{FilesystemEvent, NotifyAccessModel},
    overlay::OverlayAccessModel,
};

/// Handle to a file in [`Vfs`]
///
/// Most functions in rust-analyzer use this when they need to refer to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

pub trait AccessModel {
    type RealPath: Hash + Eq + PartialEq + for<'a> From<&'a Path>;

    fn clear(&mut self) {}

    fn mtime(&self, src: &Path) -> FileResult<SystemTime>;

    fn is_file(&self, src: &Path) -> FileResult<bool>;

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath>;

    fn content(&self, src: &Path) -> FileResult<Bytes>;
}

type FileQuery<T> = QueryRef<T, FileError>;

/// Holds canonical data for all paths pointing to the same entity.
pub struct PathSlot {
    idx: FileId,
    sampled_path: once_cell::sync::OnceCell<PathBuf>,
    mtime: FileQuery<SystemTime>,
    source: FileQuery<Source>,
    buffer: FileQuery<Bytes>,
}

impl PathSlot {
    pub fn new(idx: FileId) -> Self {
        PathSlot {
            idx,
            sampled_path: once_cell::sync::OnceCell::new(),
            mtime: FileQuery::default(),
            source: FileQuery::default(),
            buffer: FileQuery::default(),
        }
    }
}

type VfsAccessModel<M> = CachedAccessModel<OverlayAccessModel<NotifyAccessModel<M>>, Source>;

pub struct Vfs<M: AccessModel + Sized> {
    lifetime_cnt: u64,
    // access_model: TraceAccessModel<VfsAccessModel<M>>,
    // we add notify access model here since notify access model doesn't introduce overheads
    access_model: VfsAccessModel<M>,
    path_interner: Mutex<PathInterner<<M as AccessModel>::RealPath, u64>>,

    path2slot: RwLock<HashMap<Arc<OsStr>, FileId>>,
    src2file_id: RwLock<HashMap<TypstFileId, FileId>>,
    pub slots: AppendOnlyVec<PathSlot>,
    pub do_reparse: bool,
}

impl<M: AccessModel + Sized> Vfs<M> {
    pub fn new(access_model: M) -> Self {
        let access_model = NotifyAccessModel::new(access_model);
        let access_model = OverlayAccessModel::new(access_model);
        let access_model = CachedAccessModel::new(access_model);
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

    /// Reset the source manager.
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

    pub fn reset_shadow(&mut self) {
        self.access_model.inner().clear_shadow();
    }

    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.do_reparse = do_reparse;
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        self.slots.len() * core::mem::size_of::<PathSlot>()
    }

    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        let path = path.clean();
        self.path2slot.read().get(path.as_os_str()).copied()
    }

    /// Check whether a path is related to a source.
    /// Note: this does not check the canonical path, but only the normalized
    /// one.
    pub fn dependant(&self, path: &Path) -> bool {
        let path = path.clean();
        self.path2slot.read().contains_key(path.as_os_str())
    }

    /// Get all the files in the VFS.
    pub fn iter_dependencies(&self) -> impl Iterator<Item = (&Path, SystemTime)> {
        self.slots.iter().map(|slot| {
            let dep_path = slot.sampled_path.get().unwrap();
            let dep_mtime = slot
                .mtime
                .compute(|| Err(other_reason("vfs: uninitialized")))
                .unwrap();

            (dep_path.as_path(), *dep_mtime)
        })
    }

    /// Get all the files in the VFS.
    pub fn iter_dependencies_dyn<'a>(&'a self, f: &mut dyn FnMut(&'a Path, instant::SystemTime)) {
        for slot in self.slots.iter() {
            let dep_path = slot.sampled_path.get().unwrap();
            let dep_mtime = slot
                .mtime
                .compute(|| Err(other_reason("vfs: uninitialized")))
                .unwrap();

            f(dep_path.as_path(), *dep_mtime)
        }
    }

    /// File path corresponding to the given `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if the id is not present in the `Vfs`.
    pub fn file_path(&self, file_id: FileId) -> &Path {
        self.slots[file_id.0 as usize].sampled_path.get().unwrap()
    }

    /// Read a file.
    fn read(&self, path: &Path) -> FileResult<Bytes> {
        if self.access_model.is_file(path)? {
            self.access_model.content(path)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    /// Get or insert a slot for a path. All paths pointing to the same entity
    /// will share the same slot.
    ///
    /// - If `path` does not exists in the `Vfs`, allocate a new id for it,
    ///   associated with a
    /// deleted file;
    /// - Else, returns `path`'s id.
    ///
    /// Does not record a change.
    fn get_real_slot(&self, origin_path: &Path) -> FileResult<&PathSlot> {
        let real_path = self.access_model.real_path(origin_path)?;

        let mut path_interner = self.path_interner.lock();
        let (file_id, _) = path_interner.intern(real_path, self.lifetime_cnt);
        let idx = file_id.0 as usize;
        for i in self.slots.len()..idx + 1 {
            self.slots.push(PathSlot::new(FileId(i as u32)));
        }

        let slot = &self.slots[idx];
        slot.sampled_path.get_or_init(|| origin_path.to_path_buf());
        Ok(&self.slots[idx])
    }

    /// Insert a new source into the source manager.
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

    /// Get source by id.
    pub fn source(&self, file_id: TypstFileId) -> FileResult<Source> {
        let f = *self.src2file_id.read().get(&file_id).ok_or_else(|| {
            FileError::NotFound({
                // Path with package name
                let path_repr = file_id
                    .package()
                    .and_then(|pkg| file_id.vpath().resolve(Path::new(&pkg.to_string())));

                // Path without package name
                path_repr.unwrap_or_else(|| file_id.vpath().as_rootless_path().to_owned())
            })
        })?;

        self.slots[f.0 as usize]
            .source
            // the value should be computed
            .compute_ref(|| Err(other_reason("vfs: not computed source")))
            .cloned()
    }

    /// Get source id by path.
    /// This function will not check whether the path exists.
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

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path, source_id: TypstFileId) -> FileResult<Source> {
        self.resolve_with_f(path, source_id, || {
            if !self.do_reparse {
                let instant = instant::Instant::now();

                let content = self.read(path)?;
                let content = from_utf8_or_bom(&content)?.to_owned();
                let res = Ok(Source::new(source_id, content));

                println!("parse: {:?} {:?}", path, instant.elapsed());
                return res;
            }
            if self.access_model.is_file(path)? {
                Ok(self
                    .access_model
                    .read_all_diff(path, |x, y| reparse(source_id, x, y))?)
            } else {
                Err(FileError::IsDirectory)
            }
        })
    }

    pub fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.access_model.inner().add_file(path.into(), content);

        Ok(())
    }

    pub fn remove_shadow(&self, path: &Path) {
        self.access_model.inner().remove_file(path);
    }

    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.access_model.inner_mut().inner_mut().notify(event);
    }

    pub fn file(&self, path: &Path) -> FileResult<Bytes> {
        let slot = self.slot(path)?;

        let buffer = slot.buffer.compute(|| self.read(path))?;
        Ok(buffer.clone())
    }
}

fn from_utf8_or_bom(buf: &[u8]) -> FileResult<&str> {
    Ok(std::str::from_utf8(if buf.starts_with(b"\xef\xbb\xbf") {
        // remove UTF-8 BOM
        &buf[3..]
    } else {
        // Assume UTF-8
        buf
    })?)
}

fn other_reason(err: &str) -> FileError {
    FileError::Other(Some(err.into()))
}
