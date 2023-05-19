use std::{collections::HashMap, ffi::OsStr, path::Path, sync::Arc, time::SystemTime};

use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use typst::{
    diag::{FileError, FileResult},
    util::Buffer,
};
use typst_ts_core::QueryRef;

use super::AccessModel;

struct FileCache {
    lifetime_cnt: usize,
    mtime: SystemTime,
    is_file: QueryRef<bool, FileError>,
    read_all: QueryRef<Buffer, FileError>,
}

pub struct CachedAccessModel<Inner: AccessModel> {
    inner: Inner,
    lifetime_cnt: usize,
    path_results: RwLock<HashMap<Arc<OsStr>, FileCache>>,
}

impl<Inner: AccessModel> CachedAccessModel<Inner> {
    pub fn new(inner: Inner) -> Self {
        CachedAccessModel {
            inner,
            lifetime_cnt: 0,
            path_results: RwLock::new(HashMap::new()),
        }
    }
}

impl<Inner: AccessModel> CachedAccessModel<Inner> {
    fn mtime_inner(&self, src: &Path) -> FileResult<SystemTime> {
        self.inner.mtime(src)
    }

    fn cache_entry<T>(
        &self,
        src: &Path,
        cb: impl FnOnce(&FileCache) -> FileResult<T>,
    ) -> FileResult<T> {
        let path_key = src.as_os_str();
        let path_results = self.path_results.upgradable_read();
        let entry = path_results.get(path_key);
        let new_mtime = if let Some(entry) = entry {
            if entry.lifetime_cnt == self.lifetime_cnt {
                return cb(entry);
            }

            let mtime = self.mtime_inner(src)?;
            if mtime == entry.mtime {
                return cb(entry);
            }

            mtime
        } else {
            self.mtime_inner(src)?
        };

        let mut path_results = RwLockUpgradableReadGuard::upgrade(path_results);

        path_results.insert(
            path_key.into(),
            FileCache {
                lifetime_cnt: self.lifetime_cnt,
                mtime: new_mtime,
                is_file: QueryRef::default(),
                read_all: QueryRef::default(),
            },
        );

        drop(path_results);
        let path_results = self.path_results.read();
        cb(path_results.get(path_key).unwrap())
    }
}

impl<Inner: AccessModel> AccessModel for CachedAccessModel<Inner> {
    type RealPath = Inner::RealPath;

    fn clear(&mut self) {
        self.lifetime_cnt += 1;
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

    fn read_all(&self, src: &Path) -> FileResult<Buffer> {
        self.cache_entry(src, |entry| {
            let data = entry.read_all.compute(|| self.inner.read_all(src))?;
            Ok(data.clone())
        })
    }
}
