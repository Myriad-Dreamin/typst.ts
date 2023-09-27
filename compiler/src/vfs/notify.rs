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

    pub fn may_insert(&mut self, v: Option<(PathBuf, NotifyFile)>) {
        if let Some(v) = v {
            self.inserts.push(v);
        }
    }

    pub fn may_extend(&mut self, v: Option<impl Iterator<Item = (PathBuf, NotifyFile)>>) {
        if let Some(v) = v {
            self.inserts.extend(v);
        }
    }
}

#[derive(Debug)]
pub enum MemoryEvent {
    Sync(FileChangeSet),
    Update(FileChangeSet),
}

#[derive(Debug)]
pub struct UpstreamUpdateEvent {
    pub invalidates: Vec<PathBuf>,
    pub opaque: Box<dyn std::any::Any + Send>,
}

#[derive(Debug)]
pub enum FilesystemEvent {
    Update(FileChangeSet),
    UpstreamUpdate {
        changeset: FileChangeSet,
        upstream_event: Option<UpstreamUpdateEvent>,
    },
}

#[derive(Debug)]
pub enum NotifyMessage {
    /// override all dependencies
    SyncDependency(Vec<PathBuf>),
    /// upstream invalidation This is very important to make some atomic changes
    ///
    /// Example:
    /// ```plain
    ///   /// Receive memory event
    ///   let event: MemoryEvent = retrieve();
    ///   let invalidates = event.invalidates();
    ///
    ///   /// Send memory change event to [`NotifyActor`]
    ///   let event = Box::new(event);
    ///   self.send(NotifyMessage::UpstreamUpdate{ invalidates, opaque: event });
    ///
    ///   /// Wait for [`NotifyActor`] to finish
    ///   let fs_event = self.fs_notify.block_receive();
    ///   let event: MemoryEvent = fs_event.opaque.downcast().unwrap();
    ///
    ///   /// Apply changes
    ///   self.lock();
    ///   update_memory(event);
    ///   apply_fs_changes(fs_event.changeset);
    ///   self.unlock();
    /// ```
    UpstreamUpdate(UpstreamUpdateEvent),
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
            FilesystemEvent::UpstreamUpdate { changeset, .. }
            | FilesystemEvent::Update(changeset) => {
                for path in changeset.removes {
                    self.files.remove(&path);
                }

                for (path, contents) in changeset.inserts {
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
