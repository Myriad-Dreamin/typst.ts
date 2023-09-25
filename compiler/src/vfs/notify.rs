use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use typst::diag::{FileError, FileResult};
use typst_ts_core::Bytes;

use crate::vfs::AccessModel;

#[derive(Debug)]
pub enum NotifyMessage {
    /// override all dependencies
    SyncDependency(Vec<PathBuf>),
}

pub struct FileChangeSet {
    pub insert: Vec<(PathBuf, FileResult<(std::time::SystemTime, Bytes)>)>,
    pub remove: Vec<PathBuf>,
}

pub enum FilesystemEvent {
    Changed(FileChangeSet),
    CancelChanged,
}

pub struct NotifyAccessModel<M: AccessModel> {
    files: HashMap<PathBuf, FileResult<(std::time::SystemTime, Bytes)>>,
    pub inner: M,
    has_undeterminted_state: (bool, bool),
}

impl<M: AccessModel> NotifyAccessModel<M> {
    pub fn new(inner: M) -> Self {
        Self {
            files: HashMap::new(),
            inner,
            has_undeterminted_state: (false, false),
        }
    }

    pub fn notify(&mut self, event: FilesystemEvent) {
        match event {
            FilesystemEvent::Changed(files) => {
                let mut has_undeterminted_state = false;

                for removed in files.remove {
                    self.files.remove(&removed);
                }

                for (path, contents) in files.insert {
                    // peek state
                    has_undeterminted_state = has_undeterminted_state
                        || match contents.as_ref() {
                            Ok(contents) => contents.1.is_empty(),
                            Err(err) => {
                                matches!(
                                    err,
                                    FileError::NotFound(..)
                                        | FileError::Other(..)
                                        | FileError::InvalidUtf8
                                )
                            }
                        };

                    self.files.insert(path, contents);
                }

                let prev = self.has_undeterminted_state.1;
                self.has_undeterminted_state = (prev, has_undeterminted_state);
            }
            FilesystemEvent::CancelChanged => {}
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
