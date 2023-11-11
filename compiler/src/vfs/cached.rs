use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Arc};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use typst::diag::{FileError, FileResult};

use typst_ts_core::{Bytes, QueryRef};

use crate::{vfs::from_utf8_or_bom, Time};

use super::AccessModel;

/// incrementally query a value from a self holding state
type IncrQueryRef<S, E> = QueryRef<S, E, Option<S>>;

/// Holds the cached data of a single file
#[derive(Debug)]
pub struct CacheEntry<S> {
    /// The last lifetime count when the cache is updated
    last_access_lifetime: usize,
    /// The cached mtime of the file
    mtime: Time,
    /// Whether the file is a file, lazily triggered when mtime is changed
    is_file: QueryRef<bool, FileError>,
    /// The content of the file, lazily triggered when mtime is changed
    read_all: QueryRef<Bytes, FileError>,
    /// The incremental state of the source, lazily triggered when mtime is
    /// changed
    source_state: IncrQueryRef<S, FileError>,
}

/// Provides general cache to file access.
#[derive(Debug)]
pub struct CachedAccessModel<Inner: AccessModel, C> {
    /// The underlying access model for real file access
    inner: Inner,
    /// The lifetime count which resembles [`crate::vfs::Vfs::lifetime_cnt`]
    ///
    /// Note: The lifetime counter is incremented on resetting vfs.
    lifetime_cnt: usize,
    /// The cache entries for each paths
    cache_entries: RwLock<HashMap<Arc<OsStr>, CacheEntry<C>>>,
}

impl<Inner: AccessModel, C> CachedAccessModel<Inner, C> {
    /// Create a new [`CachedAccessModel`] with the given inner access model
    pub fn new(inner: Inner) -> Self {
        CachedAccessModel {
            inner,
            lifetime_cnt: 1,
            cache_entries: RwLock::new(HashMap::new()),
        }
    }

    /// Get the inner access model
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Get the mutable reference to the inner access model
    pub fn inner_mut(&mut self) -> &mut Inner {
        &mut self.inner
    }
}

impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
    fn mtime_inner(&self, src: &Path) -> FileResult<Time> {
        self.inner.mtime(src)
    }

    fn cache_entry<T>(
        &self,
        src: &Path,
        cb: impl FnOnce(&CacheEntry<C>) -> FileResult<T>,
    ) -> FileResult<T> {
        let path_key = src.as_os_str();
        let path_results = self.cache_entries.upgradable_read();
        let entry = path_results.get(path_key);
        let (new_mtime, prev_to_diff) = if let Some(entry) = entry {
            if entry.last_access_lifetime == self.lifetime_cnt {
                return cb(entry);
            }

            let mtime = self.mtime_inner(src)?;
            if mtime == entry.mtime {
                return cb(entry);
            }

            (
                mtime,
                entry
                    .source_state
                    .get_uninitialized()
                    .as_ref()
                    .and_then(|e| e.clone().ok()),
            )
        } else {
            (self.mtime_inner(src)?, None)
        };

        let mut path_results = RwLockUpgradableReadGuard::upgrade(path_results);

        path_results.insert(
            path_key.into(),
            CacheEntry {
                last_access_lifetime: self.lifetime_cnt,
                mtime: new_mtime,
                is_file: QueryRef::default(),
                read_all: QueryRef::default(),
                source_state: QueryRef::with_context(prev_to_diff),
            },
        );

        drop(path_results);
        let path_results = self.cache_entries.read();
        cb(path_results.get(path_key).unwrap())
    }
}

impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
    /// This is not a common interface for access model, but it is used for vfs
    /// incremental parsing.
    pub fn read_all_diff(
        &self,
        src: &Path,
        compute: impl FnOnce(Option<C>, String) -> FileResult<C>,
    ) -> FileResult<C> {
        self.cache_entry(src, |entry| {
            let data = entry
                .source_state
                .compute_with_context_ref(|prev_to_diff| {
                    let data = entry.read_all.compute(|| self.inner.content(src))?;
                    let text = from_utf8_or_bom(&data)?.to_owned();
                    compute(prev_to_diff, text)
                })?;

            let t = data.clone();
            Ok(t)
        })
    }
}

impl<Inner: AccessModel, C: Clone> AccessModel for CachedAccessModel<Inner, C> {
    type RealPath = Inner::RealPath;

    fn clear(&mut self) {
        self.lifetime_cnt += 1;

        let mut path_results = self.cache_entries.write();
        let new_lifetime = self.lifetime_cnt;
        path_results.retain(|_, v| new_lifetime - v.last_access_lifetime <= 30);
    }

    fn mtime(&self, src: &Path) -> FileResult<Time> {
        self.cache_entry(src, |entry| Ok(entry.mtime))
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        self.cache_entry(src, |entry| {
            entry
                .is_file
                .compute(|| self.inner.is_file(src))
                .map(|q| *q)
        })
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        // todo: cache real path
        self.inner.real_path(src)
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        self.cache_entry(src, |entry| {
            let data = entry.read_all.compute(|| self.inner.content(src));
            Ok(data?.clone())
        })
    }
}
