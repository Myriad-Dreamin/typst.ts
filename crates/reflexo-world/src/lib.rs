// use std::sync::Arc;

use core::fmt;
use std::sync::Arc;

use nohash_hasher::IntMap;
use parking_lot::{Mutex, RwLock};
use reflexo::{hash::FxDashMap, ImmutPath, QueryRef};
use reflexo_vfs::{Bytes, FileId, FsProvider, Time, TypstFileId};
use rustc_hash::FxHashMap;
use typst::{
    diag::{FileError, FileResult},
    syntax::Source,
};

pub trait MergeCache: Sized {
    fn merge(self, _other: Self) -> Self {
        self
    }
}

/// Holds canonical data for all paths pointing to the same entity.
#[derive(Debug)]
pub struct FileSlot<T> {
    pub idx: FileId,
    pub cache: T,
}

pub struct SharedState<T> {
    pub committed_revision: usize,
    // The cache entries for each paths
    cache_entries: RwLock<FxDashMap<FileId, T>>,
}

impl<T> Default for SharedState<T> {
    fn default() -> Self {
        SharedState {
            committed_revision: 0,
            cache_entries: RwLock::new(FxDashMap::default()),
        }
    }
}

impl<T> SharedState<T> {
    fn gc(&mut self) {}

    pub fn collect(state: &State<T>)
    where
        T: MergeCache,
    {
        let mut s = state.shared.write();
        state.commit_impl(&mut s);
        let _ = s.cache_entries;
        s.gc();
    }
}

pub struct State<T> {
    pub revision: usize,
    // mutex int map
    pub slots: Mutex<IntMap<TypstFileId, FileSlot<T>>>,
    shared: Arc<RwLock<SharedState<T>>>,
}

impl<T> State<T> {
    pub fn commit_impl(&self, state: &mut SharedState<T>)
    where
        T: MergeCache,
    {
        if self.revision < state.committed_revision {
            return;
        }

        state.gc();
    }
}

/// incrementally query a value from a self holding state
type IncrQueryRef<S, E> = QueryRef<S, E, Option<S>>;

type FileQuery<T> = QueryRef<T, FileError>;
type IncrFileQuery<T> = IncrQueryRef<T, FileError>;

#[derive(Default)]
pub struct GlobalSourceCache {
    pub mtime: FileQuery<Time>,
    pub source: FileQuery<Source>,
    pub buffer: FileQuery<Bytes>,
}

struct LocalSourceCache {
    last_access_lifetime: usize,
    fid: FileId,
    source: IncrFileQuery<Source>,
    buffer: FileQuery<Bytes>,
}

impl Default for LocalSourceCache {
    fn default() -> Self {
        LocalSourceCache {
            last_access_lifetime: 0,
            fid: FileId(0),
            source: IncrFileQuery::with_context(None),
            buffer: FileQuery::default(),
        }
    }
}

pub struct SourceDb {
    lifetime_cnt: usize,
    pub shared: Arc<RwLock<SharedState<GlobalSourceCache>>>,
    /// Map from typst global file id to a local file id.
    // src2file_id: RwLock<HashMap<TypstFileId, FileId>>,
    /// The slots for all the files during a single lifecycle.
    slots: Mutex<FxHashMap<TypstFileId, LocalSourceCache>>,
    /// Whether to reparse the file when it is changed.
    /// Default to `true`.
    pub do_reparse: bool,
}

impl Default for SourceDb {
    fn default() -> Self {
        SourceDb {
            lifetime_cnt: 1,
            shared: Arc::new(RwLock::new(SharedState::default())),
            slots: Mutex::new(FxHashMap::default()),
            do_reparse: true,
        }
    }
}

impl fmt::Debug for SourceDb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceDb").finish()
    }
}

impl SourceDb {
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
        // let mut w = self.slots.len() * core::mem::size_of::<PathSlot>();
        // w += self.path2slot.read().capacity() * 256;
        // w += self.src2file_id.read().capacity() * 16;
        // w += self
        //     .slots
        //     .iter()
        //     .map(|slot| {
        //         slot.source
        //             .get_uninitialized()
        //             .and_then(|e| e.as_ref().ok())
        //             .map_or(16, |e| e.text().len() * 8)
        //             + slot .buffer .get_uninitialized() .and_then(|e|
        //               e.as_ref().ok()) .map_or(16, |e| e.len())
        //     })
        //     .sum::<usize>();

        // w
        todo!()
    }

    /// Get all the files that are currently in the VFS.
    ///
    /// This is typically corresponds to the file dependencies of a single
    /// compilation.
    ///
    /// When you don't reset the vfs for each compilation, this function will
    /// still return remaining files from the previous compilation.
    pub fn iter_dependencies_dyn<'a>(
        &'a self,
        p: &'a impl FsProvider,
        f: &mut dyn FnMut(ImmutPath),
    ) {
        for slot in self.slots.lock().iter() {
            f(p.file_path(slot.1.fid));
        }
    }

    /// Get file content by path.
    pub fn file(&self, id: TypstFileId, fid: FileId, p: &impl FsProvider) -> FileResult<Bytes> {
        self.slot(id, fid, |slot| slot.buffer.compute(|| p.read(fid)).cloned())
    }

    /// Get source content by path and assign the source with a given typst
    /// global file id.
    ///
    /// See `Vfs::resolve_with_f` for more information.
    pub fn source(&self, id: TypstFileId, fid: FileId, p: &impl FsProvider) -> FileResult<Source> {
        self.slot(id, fid, |slot| {
            slot.source
                .compute_with_context(|prev| {
                    if !p.is_file(fid)? {
                        return Err(FileError::IsDirectory);
                    }

                    let content = p.read(fid)?;
                    let next = from_utf8_or_bom(&content)?.to_owned();

                    // otherwise reparse the source
                    match prev {
                        Some(mut source) if self.do_reparse => {
                            source.replace(&next);
                            Ok(source)
                        }
                        // Return a new source if we don't have a reparse feature or no prev
                        _ => Ok(Source::new(id, next)),
                    }
                })
                .cloned()
        })
    }

    // /// Get or insert a slot for a path. All paths pointing to the same entity
    // /// will share the same slot.
    // ///
    // /// - If `path` does not exists in the `Vfs`, allocate a new id for it,
    // ///   associated with a deleted file;
    // /// - Else, returns `path`'s id.
    // ///
    // /// Does not record a change.
    // fn get_real_slot(&self, file_id: FileId) -> FileResult<&PathSlot> {
    //     // // If we cannot get the real path, keep the origin path syntactically.
    //     // let real_path = self
    //     //     .access_model
    //     //     .real_path(origin_path)
    //     //     .unwrap_or_else(|_| origin_path.into());

    //     // let mut path_interner = self.path_interner.lock();
    //     // let (file_id, _) = path_interner.intern(real_path, self.lifetime_cnt);
    //     let idx = file_id.0 as usize;
    //     for i in self.slots.len()..idx + 1 {
    //         self.slots.push(PathSlot::default());
    //     }

    //     let slot = &self.slots[idx];
    //     // slot.sampled_path.get_or_init(|| origin_path.into());
    //     Ok(&self.slots[idx])
    // }

    /// Insert a new slot into the vfs.
    fn slot<T>(&self, id: TypstFileId, fid: FileId, f: impl FnOnce(&LocalSourceCache) -> T) -> T {
        // fast path for already inserted paths
        // let path2slot = self.path2slot.upgradable_read();
        // if let Some(slot) = path2slot.get(origin_path.as_os_str()) {
        //     return Ok(&self.slots[slot.0 as usize]);
        // }

        // get slot for the path
        // let slot = self.get_real_slot(origin_path)?;

        let mut slot = self.slots.lock();
        let slot = slot.entry(id).or_insert_with(|| LocalSourceCache {
            last_access_lifetime: self.lifetime_cnt,
            fid,
            source: IncrFileQuery::with_context(None),
            buffer: FileQuery::default(),
        });

        if slot.last_access_lifetime != self.lifetime_cnt {
            let prev_source = slot.source.get_uninitialized().cloned();
            let source = prev_source.transpose().ok().flatten();

            *slot = LocalSourceCache {
                last_access_lifetime: self.lifetime_cnt,
                fid,
                source: IncrFileQuery::with_context(source),
                ..Default::default()
            }
        }

        // insert the slot into the path2slot map
        // note: path aliases will share the same slot
        // let mut path2slot = RwLockUpgradableReadGuard::upgrade(path2slot);
        // let inserted = path2slot.insert(origin_path.as_os_str().into(), slot.idx);
        // assert!(inserted.is_none(), "slot already inserted");

        // let normalized = origin_path.clean();
        // if path2slot.get(normalized.as_os_str()).is_none() {
        //     let inserted = path2slot.insert(normalized.as_os_str().into(), slot.idx);
        //     assert!(inserted.is_none(), "slot already inserted");
        // }

        f(slot)
    }

    pub fn reset(&mut self) {
        self.lifetime_cnt += 1;
        let new_lifetime = self.lifetime_cnt;
        self.slots
            .get_mut()
            .retain(|_, v| new_lifetime - v.last_access_lifetime <= 30);
    }
}

pub struct FontDb {}
pub struct PackageDb {}

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
