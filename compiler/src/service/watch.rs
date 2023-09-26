//! An implementation of `loader::Handle`, based on `walkdir` and `notify`.
//!
//! The file watching bits here are untested and quite probably buggy. For this
//! reason, by default we don't watch files and rely on editor's file watching
//! capabilities.
//!
//! Hopefully, one day a reliable file watching/walking crate appears on
//! crates.io, and we can reduce this to trivial glue code.

use std::{collections::HashMap, fs, path::PathBuf};

// use crossbeam_channel::{never, select, unbounded, Receiver, Sender};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

use typst::diag::FileResult;
use typst_ts_core::Bytes;

use crate::{
    time::SystemTime,
    vfs::{
        notify::{FileChangeSet, FilesystemEvent, NotifyFile, NotifyMessage},
        system::SystemAccessModel,
        AccessModel,
    },
};

type NotifyEvent = notify::Result<notify::Event>;
type FileEntry = (/* key */ PathBuf, /* value */ NotifyFile);

struct WatchedEntry {
    // todo: generalize lifetime
    lifetime: usize,
    candidate_paths: Vec<PathBuf>,
    prev: Option<FileResult<(SystemTime, Bytes)>>,
}

impl WatchedEntry {
    fn same_prev(&self, content: &mut FileResult<(SystemTime, Bytes)>) -> bool {
        match (&self.prev, content) {
            (Some(Err(..)), Err(..)) => {
                // Both are errors, so they are the same.
                true
            }
            (Some(Ok((pt, prev))), Ok((nt, content))) => {
                // Both are Ok, so compare the content.
                if prev == content {
                    return true;
                }

                if nt == pt {
                    *nt = pt.checked_add(std::time::Duration::from_micros(1)).unwrap();
                    log::info!(
                        "same content but mtime is different...: {:?}",
                        self.candidate_paths
                    );
                }
                false
            }
            _ => false,
        }
    }
}

// Drop order is significant.
pub struct NotifyActor {
    inner: SystemAccessModel,
    lifetime: usize,
    watch: Option<()>,
    sender: mpsc::UnboundedSender<FilesystemEvent>,
    // accessing_files: HashMap<PathBuf, same_file::Handle>,
    watched_entries: HashMap<same_file::Handle, WatchedEntry>,
    watcher: Option<(RecommendedWatcher, mpsc::UnboundedReceiver<NotifyEvent>)>,
}

impl NotifyActor {
    fn new(sender: mpsc::UnboundedSender<FilesystemEvent>) -> NotifyActor {
        NotifyActor {
            inner: SystemAccessModel,
            lifetime: 0,
            watch: Some(()),
            sender,
            // accessing_files: HashMap::new(),
            watched_entries: HashMap::new(),
            watcher: None,
        }
    }

    fn send(&mut self, msg: FilesystemEvent) {
        self.sender.send(msg).unwrap();
    }

    async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<NotifyMessage>) {
        #[derive(Debug)]
        enum ActorEvent {
            // ReCheckEmptyFile(PathBuf),
            Message(NotifyMessage),
            NotifyEvent(NotifyEvent),
        }

        loop {
            let event = match self.watcher.as_mut() {
                Some((.., watcher_receiver)) => tokio::select! {
                    Some(it) = inbox.recv() => Some(ActorEvent::Message(it)),
                    Some(it) = watcher_receiver.recv() => Some(ActorEvent::NotifyEvent(it)),
                },
                None => inbox.recv().await.map(ActorEvent::Message),
            };

            let Some(event) = event else {
                return;
            };

            // log::info!("vfs-notify event {event:?}");

            match event {
                ActorEvent::Message(NotifyMessage::SyncDependency(paths)) => {
                    self.sync_dependency(paths).await;
                }
                ActorEvent::NotifyEvent(event) => {
                    self.notify_event(event).await;
                }
            }
        }
    }

    async fn sync_dependency(&mut self, paths: Vec<PathBuf>) {
        self.lifetime += 1;

        let changeset = FileChangeSet::default();

        // Remove old watches, if any.
        self.watcher = None;
        if self.watch.is_some() {
            match &mut self.watcher {
                Some((old_watcher, _)) => {
                    let entries = self.watched_entries.values();
                    for path in entries.flat_map(|entry| &entry.candidate_paths) {
                        // Remove the watch if it still exists.
                        if let Err(err) = old_watcher.unwatch(path) {
                            if !matches!(err.kind, notify::ErrorKind::WatchNotFound) {
                                log::warn!("failed to unwatch: {err}");
                            }
                        }
                    }
                }
                None => {
                    let (watcher_sender, watcher_receiver) = mpsc::unbounded_channel();
                    let watcher = log_notify_error(
                        RecommendedWatcher::new(
                            move |event| {
                                let res = watcher_sender.send(event);
                                if let Err(err) = res {
                                    log::warn!("error to send event: {err}");
                                }
                            },
                            Config::default(),
                        ),
                        "failed to create watcher",
                    );
                    self.watcher = watcher.map(|it| (it, watcher_receiver));
                }
            }
        }

        // Update watched entries.
        let mut insert_entries = vec![];
        for path in paths.into_iter() {
            let watch = self.watch.is_some(); // paths.watch.contains(&i);
            let meta = path.metadata().unwrap();

            if watch {
                // Remove old watches, if any.
                let handle = same_file::Handle::from_path(path.clone()).unwrap();
                let entry = self
                    .watched_entries
                    .entry(handle)
                    .and_modify(|watch_entry| {
                        // watch_entry.candidate_paths.push(watch_entry.clone());
                        if !watch_entry.candidate_paths.iter().any(|it| **it == path) {
                            watch_entry.candidate_paths.push(path.clone());
                        }
                        watch_entry.lifetime = self.lifetime;
                    })
                    .or_insert_with(|| WatchedEntry {
                        lifetime: self.lifetime,
                        candidate_paths: vec![path.to_owned()],
                        prev: None,
                    });

                // Watch the file again if it's not a directory.
                if !meta.is_dir() {
                    if let Some((watcher, _)) = &mut self.watcher {
                        log_notify_error(
                            watcher.watch(path.as_ref(), RecursiveMode::NonRecursive),
                            "failed to watch",
                        );

                        let mut watched = self
                            .inner
                            .content(&path)
                            .map(|e| (meta.modified().unwrap(), e));
                        if !entry.same_prev(&mut watched) {
                            entry.prev = Some(watched.clone());
                            insert_entries.push((path, watched));
                        }
                    } else {
                        unreachable!()
                    }
                }
            } else {
                let watched = self
                    .inner
                    .content(&path)
                    .map(|e| (meta.modified().unwrap(), e));
                insert_entries.push((path, watched));
            }
        }

        let mut remove_entries = vec![];
        self.watched_entries.retain(|_, entry| {
            if self.lifetime - entry.lifetime < 30 {
                true
            } else {
                remove_entries.extend(std::mem::take(&mut entry.candidate_paths));
                false
            }
        });

        if !changeset.is_empty() {
            self.send(FilesystemEvent::Update(changeset));
        }
    }

    fn notify_entry_update(&mut self, path: PathBuf) -> Option<FileEntry> {
        let meta = fs::metadata(&path).ok()?;
        // if meta.file_type().is_dir()
        //     && self
        //         .watched_entries
        //         .iter()
        //         .any(|entry| entry.contains_dir(&path))
        // {
        //     self.watch(path);
        //     return None;
        // }

        if !meta.file_type().is_file() {
            return None;
        }

        // Check meta, path, and content

        // Get meta, real path and ignore errors
        let mtime = meta.modified().ok()?;
        let handle = same_file::Handle::from_path(&path).ok()?;

        // Find entry and continue
        let entry = self.watched_entries.get_mut(&handle)?;

        // Fast path: compare content
        let mut content = self.inner.content(&path).map(|it| (mtime, it));

        if entry.same_prev(&mut content) {
            return None;
        }
        entry.prev = Some(content.clone());

        // Slow path: trigger the compiler
        Some((path, content.into()))
    }

    async fn notify_event(&mut self, event: NotifyEvent) {
        // log::info!("notify event {event:?}");
        let event = log_notify_error(event, "failed to notify");

        // Account file updates.
        let file_update = event.map(|event| {
            let paths = event.paths.into_iter();
            let updates = paths.filter_map(|path| self.notify_entry_update(path));

            updates.collect()
        });

        // Send file updates.
        let Some(changeset) = file_update.map(FileChangeSet::new_inserts) else {
            return;
        };
        if !changeset.is_empty() {
            self.send(FilesystemEvent::Update(changeset));
        }
    }
}

#[inline]
fn log_notify_error<T>(res: notify::Result<T>, reason: &'static str) -> Option<T> {
    res.map_err(|err| log::warn!("{reason}: notify error: {}", err))
        .ok()
}

pub async fn watch_deps(
    inbox: mpsc::UnboundedReceiver<NotifyMessage>,
    mut interrupted_by_events: impl FnMut(Option<FilesystemEvent>),
) {
    // Setup file watching.
    let (tx, mut rx) = mpsc::unbounded_channel();
    let actor = NotifyActor::new(tx);

    // Watch messages to notify
    tokio::spawn(actor.run(inbox));

    // Handle events.
    log::info!("start watching files...");
    interrupted_by_events(None);
    while let Some(event) = rx.recv().await {
        interrupted_by_events(Some(event));
    }
}
