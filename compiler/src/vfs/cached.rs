use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Arc, time::SystemTime};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use typst::diag::{FileError, FileResult};

use typst_ts_core::{Bytes, QueryRef};

use crate::vfs::from_utf8_or_bom;

use super::AccessModel;

/// incrementally query a value from a self holding state
type IncrQueryRef<S, E> = QueryRef<S, E, Option<S>>;

pub struct FileCache<S> {
    lifetime_cnt: usize,
    mtime: SystemTime,
    is_file: QueryRef<bool, FileError>,
    read_all: QueryRef<Bytes, FileError>,
    source_state: IncrQueryRef<S, FileError>,
}

pub struct CachedAccessModel<Inner: AccessModel, C> {
    inner: Inner,
    lifetime_cnt: usize,
    path_results: RwLock<HashMap<Arc<OsStr>, FileCache<C>>>,
}

impl<Inner: AccessModel, C> CachedAccessModel<Inner, C> {
    pub fn new(inner: Inner) -> Self {
        CachedAccessModel {
            inner,
            lifetime_cnt: 0,
            path_results: RwLock::new(HashMap::new()),
        }
    }

    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
    fn mtime_inner(&self, src: &Path) -> FileResult<SystemTime> {
        self.inner.mtime(src)
    }

    fn cache_entry<T>(
        &self,
        src: &Path,
        cb: impl FnOnce(&FileCache<C>) -> FileResult<T>,
    ) -> FileResult<T> {
        let path_key = src.as_os_str();
        let path_results = self.path_results.upgradable_read();
        let entry = path_results.get(path_key);
        let (new_mtime, prev_to_diff) = if let Some(entry) = entry {
            if entry.lifetime_cnt == self.lifetime_cnt {
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
            FileCache {
                lifetime_cnt: self.lifetime_cnt,
                mtime: new_mtime,
                is_file: QueryRef::default(),
                read_all: QueryRef::default(),
                source_state: QueryRef::with_context(prev_to_diff),
            },
        );

        drop(path_results);
        let path_results = self.path_results.read();
        cb(path_results.get(path_key).unwrap())
    }
}

impl<Inner: AccessModel, C: Clone> CachedAccessModel<Inner, C> {
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

        let mut path_results = self.path_results.write();
        let new_lifetime = self.lifetime_cnt;
        path_results.retain(|_, v| new_lifetime - v.lifetime_cnt <= 30);
    }

    fn mtime(&self, src: &Path) -> FileResult<SystemTime> {
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
            let data = entry.read_all.compute(|| self.inner.content(src))?;
            Ok(data.clone())
        })
    }
}
