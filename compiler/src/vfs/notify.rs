use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use typst::diag::FileResult;
use typst_ts_core::Bytes;

use crate::vfs::AccessModel;

#[derive(Debug, Clone)]
struct NotifyFileRepr {
    mtime: instant::SystemTime,
    content: Bytes,
}

#[derive(Debug, Clone)]
pub struct NotifyFile(FileResult<NotifyFileRepr>);

impl NotifyFile {
    pub fn mtime(&self) -> FileResult<&instant::SystemTime> {
        self.0.as_ref().map(|e| &e.mtime).map_err(|e| e.clone())
    }

    pub fn content(&self) -> FileResult<&Bytes> {
        self.0.as_ref().map(|e| &e.content).map_err(|e| e.clone())
    }

    pub fn ok(&self) -> FileResult<bool> {
        self.0.as_ref().map(|_| true).map_err(|e| e.clone())
    }
}

impl From<FileResult<(instant::SystemTime, Bytes)>> for NotifyFile {
    fn from(result: FileResult<(instant::SystemTime, Bytes)>) -> Self {
        Self(result.map(|(mtime, content)| NotifyFileRepr { mtime, content }))
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileChangeSet {
    pub inserts: Vec<(PathBuf, NotifyFile)>,
    pub removes: Vec<PathBuf>,
}

impl FileChangeSet {
    pub fn is_empty(&self) -> bool {
        self.inserts.is_empty() && self.removes.is_empty()
    }

    pub fn new_removes(removes: Vec<PathBuf>) -> Self {
        Self {
            inserts: vec![],
            removes,
        }
    }

    pub fn new_inserts(inserts: Vec<(PathBuf, NotifyFile)>) -> Self {
        Self {
            inserts,
            removes: vec![],
        }
    }
}

#[derive(Debug)]
pub enum NotifyMessage {
    /// override all dependencies
    SyncDependency(Vec<PathBuf>),
}

#[derive(Debug, Clone)]
pub enum FilesystemEvent {
    Update(FileChangeSet),
}

pub struct NotifyAccessModel<M: AccessModel> {
    files: HashMap<PathBuf, NotifyFile>,
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
                for path in files.removes {
                    self.files.remove(&path);
                }

                for (path, contents) in files.inserts {
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
            return entry.mtime().cloned();
        }

        self.inner.mtime(src)
    }

    fn is_file(&self, src: &Path) -> FileResult<bool> {
        if let Some(entry) = self.files.get(src) {
            return entry.ok();
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
            return entry.content().cloned();
        }

        self.inner.content(src)
    }
}
