//! upstream of following files https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs
//!   ::path_interner.rs -> path_interner.rs
//!   ::paths.rs -> abs_path.rs
//!   ::anchored_path.rs -> path_anchored.rs
//!   ::vfs_path.rs -> path_vfs.rs

#[cfg(feature = "system")]
pub mod system;

pub mod dummy;

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
    syntax::{Source, SourceId},
    util::{Buffer, PathExt},
};
use typst_ts_core::QueryRef;

/// Handle to a file in [`Vfs`]
///
/// Most functions in rust-analyzer use this when they need to refer to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

pub trait AccessModel {
    type RealPath: Hash + Eq + PartialEq;

    fn mtime(&self, src: &Path) -> std::io::Result<std::time::SystemTime>;

    fn is_file(&self, src: &Path) -> std::io::Result<bool>;

    fn real_path(&self, src: &Path) -> std::io::Result<Self::RealPath>;

    fn read_all(&self, src: &Path, buf: &mut Vec<u8>) -> std::io::Result<usize>;
}

/// Holds canonical data for all paths pointing to the same entity.
pub struct PathSlot {
    idx: FileId,
    sampled_path: once_cell::sync::OnceCell<PathBuf>,
    source: QueryRef<Source, FileError>,
    buffer: QueryRef<Buffer, FileError>,
}

impl PathSlot {
    pub fn new(idx: FileId) -> Self {
        PathSlot {
            idx,
            sampled_path: once_cell::sync::OnceCell::new(),
            source: QueryRef::default(),
            buffer: QueryRef::default(),
        }
    }
}

pub struct Vfs<M: AccessModel + Sized> {
    access_model: M,
    path_interner: Mutex<PathInterner<<M as AccessModel>::RealPath>>,

    path2slot: RwLock<HashMap<Arc<OsStr>, FileId>>,
    pub slots: AppendOnlyVec<PathSlot>,
}

impl<M: AccessModel + Sized> Vfs<M> {
    pub fn new(access_model: M) -> Self {
        Self {
            access_model,
            path_interner: Mutex::new(PathInterner::default()),
            slots: AppendOnlyVec::new(),
            path2slot: RwLock::new(HashMap::new()),
        }
    }

    /// Reset the source manager.
    pub fn reset(&mut self) {
        self.slots = AppendOnlyVec::new();
        self.path2slot.get_mut().clear();
        self.path_interner.get_mut().clear();
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        self.slots.len() * core::mem::size_of::<PathSlot>()
    }

    /// Id of the given path if it exists in the `Vfs` and is not deleted.
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        let path = path.normalize();
        self.path2slot.read().get(path.as_os_str()).copied()
    }

    /// Check whether a path is related to a source.
    pub fn dependant(&self, path: &Path) -> bool {
        let path = path.normalize();
        self.path2slot.read().contains_key(path.as_os_str())
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
    fn read(&self, path: &Path) -> FileResult<Vec<u8>> {
        let f = |e| FileError::from_io(e, path);
        if self.access_model.is_file(path).map_err(f)? {
            let mut data = vec![];
            self.access_model.read_all(path, &mut data).map_err(f)?;
            Ok(data)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    /// Get or insert a slot for a path. All paths pointing to the same entity will share the same slot.
    ///
    /// - If `path` does not exists in the `Vfs`, allocate a new id for it, associated with a
    /// deleted file;
    /// - Else, returns `path`'s id.
    ///
    /// Does not record a change.
    fn get_real_slot(&self, origin_path: &Path) -> FileResult<&PathSlot> {
        let f = |e| FileError::from_io(e, origin_path);
        let real_path = self.access_model.real_path(origin_path).map_err(f)?;

        let mut path_interner = self.path_interner.lock();
        let file_id = path_interner.intern(real_path);
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
        assert!(matches!(inserted, None), "slot already inserted");

        let normalized = origin_path.normalize();
        if path2slot.get(normalized.as_os_str()).is_none() {
            let inserted = path2slot.insert(normalized.as_os_str().into(), slot.idx);
            assert!(matches!(inserted, None), "slot already inserted");
        }

        Ok(slot)
    }

    /// Get source by id.
    pub fn source(&self, id: SourceId) -> &Source {
        self.slots[id.into_u16() as usize]
            .source
            // the value should be computed
            .compute_ref(|| Err(FileError::Other))
            .unwrap()
    }

    /// Get source id by path.
    /// This function will not check whether the path exists.
    fn resolve_with_f<ReadContent: FnOnce() -> FileResult<String>>(
        &self,
        path: &Path,
        read: ReadContent,
    ) -> FileResult<SourceId> {
        let slot = self.slot(path)?;
        let source_id = SourceId::from_u16(slot.idx.0 as u16);

        slot.source.compute(|| {
            let text = read()?;
            Ok(Source::new(source_id, path, text))
        })?;

        Ok(source_id)
    }

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.resolve_with_f(path, || {
            let buf = self.read(path)?;
            Ok(String::from_utf8(buf)?)
        })
    }

    // todo: remove
    /// Get source id by path with memory content.
    pub fn resolve_with<P: AsRef<Path>>(&self, path: P, content: &str) -> FileResult<SourceId> {
        self.resolve_with_f(path.as_ref(), || Ok(content.to_owned()))
    }

    pub fn file(&self, path: &Path) -> FileResult<Buffer> {
        let slot = self.slot(path)?;

        let buffer = slot.buffer.compute(|| {
            let buf = self.read(path)?;
            let buf = Buffer::from(buf);
            Ok(buf)
        })?;

        Ok(buffer.clone())
    }
}
