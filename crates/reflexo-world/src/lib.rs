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

impl fmt::Debug for GlobalSourceCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GlobalSourceCache").finish()
    }
}

pub struct LocalSourceCache {
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

#[derive(Default)]
pub struct SourceDb {
    pub revision: usize,
    pub shared: Arc<RwLock<SharedState<GlobalSourceCache>>>,
    /// The slots for all the files during a single lifecycle.
    pub slots: Mutex<FxHashMap<TypstFileId, LocalSourceCache>>,
    /// Whether to reparse the file when it is changed.
    /// Default to `true`.
    pub do_reparse: bool,
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
        let mut w = self.slots.lock().len() * core::mem::size_of::<LocalSourceCache>();
        w += self
            .slots
            .lock()
            .iter()
            .map(|(_, slot)| {
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

    /// Insert a new slot into the vfs.
    fn slot<T>(&self, id: TypstFileId, fid: FileId, f: impl FnOnce(&LocalSourceCache) -> T) -> T {
        let mut slot = self.slots.lock();
        let slot = slot.entry(id).or_insert_with(|| LocalSourceCache {
            last_access_lifetime: self.revision,
            fid,
            source: IncrFileQuery::with_context(None),
            buffer: FileQuery::default(),
        });

        if slot.last_access_lifetime != self.revision {
            let prev_source = slot.source.get_uninitialized().cloned();
            let source = prev_source.transpose().ok().flatten();

            *slot = LocalSourceCache {
                last_access_lifetime: self.revision,
                fid,
                source: IncrFileQuery::with_context(source),
                ..Default::default()
            }
        }

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

impl fmt::Debug for SharedState<GlobalSourceCache> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SharedState")
            .field("committed_revision", &self.committed_revision)
            .finish()
    }
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
