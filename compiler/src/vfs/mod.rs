//! upstream of following files <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs>
//!   ::path_interner.rs -> path_interner.rs
//!   ::paths.rs -> abs_path.rs
//!   ::anchored_path.rs -> path_anchored.rs
//!   ::vfs_path.rs -> path_vfs.rs

#[cfg(feature = "browser-compile")]
pub mod browser;

#[cfg(feature = "system")]
pub mod system;

pub mod cached;
pub mod dummy;
pub mod trace;

mod path_abs;
mod path_anchored;
pub(crate) mod path_ext;
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
    syntax::{Source, SourceId},
    util::PathExt,
};

use typst_ts_core::{typst_affinite_hash, Bytes, QueryRef};

use self::{
    cached::{CachedAccessModel, FileCache},
    trace::TraceAccessModel,
};

/// Handle to a file in [`Vfs`]
///
/// Most functions in rust-analyzer use this when they need to refer to a file.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct FileId(pub u32);

/// safe because `FileId` is a newtype of `u32`
impl nohash_hasher::IsEnabled for FileId {}

pub trait AccessModel {
    type RealPath: Hash + Eq + PartialEq;

    fn clear(&mut self) {}

    fn mtime(&self, src: &Path) -> FileResult<std::time::SystemTime>;

    fn is_file(&self, src: &Path) -> FileResult<bool>;

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath>;

    fn read_all(&self, src: &Path) -> FileResult<Bytes>;
}

type FileQuery<T> = QueryRef<T, FileError>;

/// Holds canonical data for all paths pointing to the same entity.
pub struct PathSlot {
    idx: FileId,
    sampled_path: once_cell::sync::OnceCell<PathBuf>,
    mtime: FileQuery<std::time::SystemTime>,
    source: FileQuery<Arc<Source>>,
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

pub struct Vfs<M: AccessModel + Sized> {
    lifetime_cnt: u64,
    access_model: TraceAccessModel<CachedAccessModel<M, Source>>,
    path_interner: Mutex<PathInterner<<M as AccessModel>::RealPath, u64>>,

    path2slot: RwLock<HashMap<Arc<OsStr>, FileId>>,
    pub slots: AppendOnlyVec<PathSlot>,
    pub do_reparse: bool,
}

impl<M: AccessModel + Sized> Vfs<M> {
    pub fn new(access_model: M) -> Self {
        Self {
            lifetime_cnt: 0,
            access_model: TraceAccessModel::new(CachedAccessModel::new(access_model)),
            path_interner: Mutex::new(PathInterner::default()),
            slots: AppendOnlyVec::new(),
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
        self.path_interner
            .get_mut()
            .retain(|_, lifetime| new_lifetime_cnt - *lifetime <= 30);
        self.access_model.clear();
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
        let path = path.normalize();
        self.path2slot.read().get(path.as_os_str()).copied()
    }

    /// Check whether a path is related to a source.
    pub fn dependant(&self, path: &Path) -> bool {
        let path = path.normalize();
        self.path2slot.read().contains_key(path.as_os_str())
    }

    /// Get all the files in the VFS.
    pub fn iter_dependencies(&self) -> impl Iterator<Item = (&Path, std::time::SystemTime)> {
        self.slots.iter().map(|slot| {
            let dep_path = slot.sampled_path.get().unwrap();
            let dep_mtime = slot.mtime.compute(|| Err(FileError::Other)).unwrap();

            (dep_path.as_path(), *dep_mtime)
        })
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
            self.access_model.read_all(path)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    /// Read a source with diff.
    fn read_diff_source(&self, path: &Path, source_id: SourceId) -> FileResult<Arc<Source>> {
        if self.access_model.is_file(path)? {
            Ok(self
                .access_model
                .read_all_diff(path, |x, y| self.reparse(path, source_id, x, y))?)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    /// Read a source with diff.
    fn replace_diff_source(
        &self,
        path: &Path,
        source_id: SourceId,
        read: impl FnOnce(&FileCache<Source>) -> FileResult<Bytes>,
    ) -> FileResult<Arc<Source>> {
        if self.access_model.is_file(path)? {
            Ok(self
                .access_model
                .replace_diff(path, read, |x, y| self.reparse(path, source_id, x, y))?)
        } else {
            Err(FileError::IsDirectory)
        }
    }

    fn reparse(
        &self,
        path: &Path,
        source_id: SourceId,
        prev: Option<Source>,
        next: String,
    ) -> FileResult<Source> {
        let instant = std::time::Instant::now();
        println!("reparse: {:?}", path);
        use dissimilar::Chunk;
        match prev {
            Some(mut source) => {
                let prev = source.text();
                if prev == next {
                    println!("same: {:?} -> {:?}", path, typst_affinite_hash(&source));
                    Ok(source)
                } else {
                    let prev = prev.to_owned();
                    let prev_hash = typst_affinite_hash(&source);

                    let diff = dissimilar::diff(&prev, &next);

                    let elapsed = instant.elapsed();
                    let diff_instant = std::time::Instant::now();

                    let mut rev_adavance = 0;
                    let mut last_rep = false;
                    let prev_len = prev.len();
                    for op in diff.iter().rev().zip(diff.iter().rev().skip(1)) {
                        if last_rep {
                            last_rep = false;
                            continue;
                        }
                        match op {
                            (Chunk::Delete(t), Chunk::Insert(s))
                            | (Chunk::Insert(s), Chunk::Delete(t)) => {
                                println!("[{}] {} -> {}", rev_adavance, t, s);
                                rev_adavance += t.len();
                                source.edit(
                                    prev_len - rev_adavance..prev_len - rev_adavance + t.len(),
                                    s,
                                );
                                last_rep = true;
                            }
                            (Chunk::Delete(t), Chunk::Equal(e)) => {
                                println!("[{}] -- {}", rev_adavance, t);
                                rev_adavance += t.len();
                                source.edit(
                                    prev_len - rev_adavance..prev_len - rev_adavance + t.len(),
                                    "",
                                );
                                rev_adavance += e.len();
                                last_rep = true;
                            }
                            (Chunk::Insert(s), Chunk::Equal(e)) => {
                                println!("[{}] ++ {}", rev_adavance, s);
                                source.edit(prev_len - rev_adavance..prev_len - rev_adavance, s);
                                last_rep = true;
                                rev_adavance += e.len();
                            }
                            (Chunk::Equal(t), _) => {
                                rev_adavance += t.len();
                            }
                            _ => unreachable!(),
                        }
                    }

                    println!(
                        "reparse real: d:{:?} e:{:?} {:?} {:?} -> {:?}",
                        elapsed,
                        diff_instant.elapsed(),
                        path,
                        prev_hash,
                        typst_affinite_hash(&source)
                    );
                    Ok(source)
                }
            }
            None => Ok(Source::new(source_id, path, next)),
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
        assert!(matches!(inserted, None), "slot already inserted");

        let normalized = origin_path.normalize();
        if path2slot.get(normalized.as_os_str()).is_none() {
            let inserted = path2slot.insert(normalized.as_os_str().into(), slot.idx);
            assert!(matches!(inserted, None), "slot already inserted");
        }

        // prefetch a early mtime
        slot.mtime
            .compute(|| self.access_model.mtime(origin_path))?;

        Ok(slot)
    }

    /// Get source by id.
    pub fn source(&self, id: SourceId) -> &Source {
        self.slots[id.as_u16() as usize]
            .source
            // the value should be computed
            .compute_ref(|| Err(FileError::Other))
            .unwrap()
    }

    /// Get source id by path.
    /// This function will not check whether the path exists.
    fn resolve_with_f<ReadContent: FnOnce(SourceId) -> FileResult<Arc<Source>>>(
        &self,
        path: &Path,
        read: ReadContent,
    ) -> FileResult<SourceId> {
        let slot = self.slot(path)?;
        let origin_source_id = slot.idx.0;
        let source_id = SourceId::from_u16(origin_source_id as u16);

        slot.source.compute(|| {
            if origin_source_id > u16::MAX as u32 {
                panic!("source id overflow");
            }

            read(source_id)
        })?;

        Ok(source_id)
    }

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.resolve_with_f(path, |source_id| {
            if !self.do_reparse {
                let instant = std::time::Instant::now();

                let content = self.read(path)?;
                let content = from_utf8_or_bom(&content)?.to_owned();
                let res = Ok(Arc::new(Source::new(source_id, path, content)));

                println!("parse: {:?} {:?}", path, instant.elapsed());
                return res;
            }
            self.read_diff_source(path, source_id)
        })
    }

    /// Get source id by path with memory content.
    pub fn resolve_with<P: AsRef<Path>>(&self, path: P, content: &str) -> FileResult<SourceId> {
        let path = path.as_ref();
        self.resolve_with_f(path, |source_id| {
            if !self.do_reparse {
                let instant = std::time::Instant::now();

                let res = Ok(Arc::new(Source::new(source_id, path, content.to_owned())));

                println!("parse: {:?} {:?}", path, instant.elapsed());
                return res;
            }
            self.replace_diff_source(path, source_id, |_| Ok(Bytes::from(content.as_bytes())))
        })
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
