use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use typst::diag::FileResult;
use typst_ts_core::Bytes;

use crate::vfs::AccessModel;

#[derive(Debug)]
pub enum NotifyMessage {
    /// override all dependencies
    SyncDependency(Vec<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct FileChangeSet {
    pub insert: Vec<(PathBuf, FileResult<(std::time::SystemTime, Bytes)>)>,
    pub remove: Vec<PathBuf>,
}

impl FileChangeSet {
    pub fn new_remove(paths: Vec<PathBuf>) -> Self {
        Self {
            insert: vec![],
            remove: paths.into_iter().collect(),
        }
    }

    pub fn new_insert(inserts: Vec<(PathBuf, FileResult<(std::time::SystemTime, Bytes)>)>) -> Self {
        Self {
            insert: inserts,
            remove: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilesystemEvent {
    Update(FileChangeSet),
}

pub struct NotifyAccessModel<M: AccessModel> {
    files: HashMap<PathBuf, FileResult<(std::time::SystemTime, Bytes)>>,
    pub inner: M,
}

impl<M: AccessModel> NotifyAccessModel<M> {
    pub fn new(inner: M) -> Self {
        Self {
            files: HashMap::new(),
            inner,
        }
    }

    pub fn notify(&mut self, event: FilesystemEvent) {
        match event {
            FilesystemEvent::Update(files) => {
                for path in files.remove {
                    self.files.remove(&path);
                }

                for (path, contents) in files.insert {
                    self.files.insert(path, contents);
                }
            }
        }
    }
}

impl<M: AccessModel> AccessModel for NotifyAccessModel<M> {
    type RealPath = M::RealPath;

    fn mtime(&self, src: &Path) -> FileResult<crate::time::SystemTime> {
        if let Some(entry) = self.files.get(src) {
            return entry.clone().map(|e| e.0);
        }

        self.inner.mtime(src)
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        if let Some(entry) = self.files.get(src) {
            return entry.clone().map(|_| true);
        }

        self.inner.is_file(src)
    }

    fn real_path(&self, src: &Path) -> FileResult<Self::RealPath> {
        if self.files.get(src).is_some() {
            return Ok(src.into());
        }

        self.inner.real_path(src)
    }

    fn content(&self, src: &Path) -> FileResult<Bytes> {
        if let Some(entry) = self.files.get(src) {
            return entry.clone().map(|e| e.1);
        }

        self.inner.content(src)
    }
}
