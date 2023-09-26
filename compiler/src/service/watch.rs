//! An implementation of `watch_deps` using `notify` crate.
//!
//! The file watching bits here are untested and quite probably buggy. For this
//! reason, by default we don't watch files and rely on editor's file watching
//! capabilities.
//!
//! Hopefully, one day a reliable file watching/walking crate appears on
//! crates.io, and we can reduce this to trivial glue code.

use std::{collections::HashMap, fs, path::PathBuf};

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use typst::diag::{FileError, FileResult};

use typst_ts_core::Bytes;

use crate::vfs::{
    notify::{FileChangeSet, FilesystemEvent, NotifyFile, NotifyMessage},
    system::SystemAccessModel,
    AccessModel,
};

type WatcherPair = (RecommendedWatcher, mpsc::UnboundedReceiver<NotifyEvent>);
type NotifyEvent = notify::Result<notify::Event>;
type FileEntry = (/* key */ PathBuf, /* value */ NotifyFile);
type NotifyFilePair = FileResult<(instant::SystemTime, Bytes)>;

#[derive(Debug)]
enum WatchState {
    Fresh,
    EmptyOrRemoval {
        recheck_at: usize,
        payload: NotifyFilePair,
    },
}

impl Default for WatchState {
    fn default() -> Self {
        Self::Fresh
    }
}

struct WatchedEntry {
    // todo: generalize lifetime
    lifetime: usize,
    state: WatchState,
    candidate_paths: Vec<PathBuf>,
    prev: Option<NotifyFilePair>,
}

#[derive(Debug)]
struct UndeterminedNotifyEvent {
    at_realtime: instant::Instant,
    at_logical_tick: usize,
    path: PathBuf,
}

// Drop order is significant.
pub struct NotifyActor {
    inner: SystemAccessModel,
    lifetime: usize,
    watch: Option<()>,
    sender: mpsc::UnboundedSender<FilesystemEvent>,

    undetermined_send: mpsc::UnboundedSender<UndeterminedNotifyEvent>,
    undetermined_recv: mpsc::UnboundedReceiver<UndeterminedNotifyEvent>,

    // accessing_files: HashMap<PathBuf, same_file::Handle>,
    watched_entries: HashMap<same_file::Handle, WatchedEntry>,
    watcher: Option<WatcherPair>,
}

impl NotifyActor {
    fn new(sender: mpsc::UnboundedSender<FilesystemEvent>) -> NotifyActor {
        let (undetermined_send, undetermined_recv) = mpsc::unbounded_channel();

        NotifyActor {
            inner: SystemAccessModel,
            lifetime: 1,
            watch: Some(()),
            sender,

            undetermined_send,
            undetermined_recv,

            // accessing_files: HashMap::new(),
            watched_entries: HashMap::new(),
            watcher: None,
        }
    }

    fn send(&mut self, msg: FilesystemEvent) {
        self.sender.send(msg).unwrap();
    }

    async fn get_notify_event(watcher: &mut Option<WatcherPair>) -> Option<NotifyEvent> {
        match watcher {
            Some((_, watcher_receiver)) => watcher_receiver.recv().await,
            None => None,
        }
    }

    async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<NotifyMessage>) {
        #[derive(Debug)]
        enum ActorEvent {
            // ReCheckEmptyFile(PathBuf),
            ReCheck(UndeterminedNotifyEvent),
            Message(NotifyMessage),
            NotifyEvent(NotifyEvent),
        }

        loop {
            let event = tokio::select! {
                Some(it) = inbox.recv() => Some(ActorEvent::Message(it)),
                Some(it) = Self::get_notify_event(&mut self.watcher) => Some(ActorEvent::NotifyEvent(it)),
                Some(it) = self.undetermined_recv.recv() => Some(ActorEvent::ReCheck(it)),
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
                    // log::info!("notify event {event:?}");
                    if let Some(event) = log_notify_error(event, "failed to notify") {
                        self.notify_event(event).await;
                    }
                }
                ActorEvent::ReCheck(event) => {
                    if let Some(stablized_change) = self.recheck_notify_event(event).await {
                        let mut changeset = FileChangeSet::default();
                        changeset.inserts.push(stablized_change);
                        self.send(FilesystemEvent::Update(changeset));
                    }
                }
            }
        }
    }

    async fn sync_dependency(&mut self, paths: Vec<PathBuf>) {
        self.lifetime += 1;

        let mut changeset = FileChangeSet::default();

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
        for path in paths.into_iter() {
            let watch = self.watch.is_some(); // paths.watch.contains(&i);
            let meta = path.metadata().unwrap();

            if watch {
                // Remove old watches, if any.
                let handle = same_file::Handle::from_path(path.clone()).unwrap();
                self.watched_entries
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
                        state: WatchState::Fresh,
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

                        changeset.may_insert(self.notify_entry_update(path.clone(), Some(meta)));
                    } else {
                        unreachable!()
                    }
                }
            } else {
                let watched = self
                    .inner
                    .content(&path)
                    .map(|e| (meta.modified().unwrap(), e));
                changeset.inserts.push((path, watched.into()));
            }
        }

        self.watched_entries.retain(|_, entry| {
            if self.lifetime - entry.lifetime < 500 {
                true
            } else {
                changeset
                    .removes
                    .extend(std::mem::take(&mut entry.candidate_paths));
                false
            }
        });

        if !changeset.is_empty() {
            self.send(FilesystemEvent::Update(changeset));
        }
    }

    async fn notify_event(&mut self, event: notify::Event) {
        // Account file updates.
        let mut changeset = FileChangeSet::default();
        for path in event.paths.into_iter() {
            changeset.may_insert(self.notify_entry_update(path.clone(), None));
        }

        // Send file updates.
        if !changeset.is_empty() {
            self.send(FilesystemEvent::Update(changeset));
        }
    }

    fn notify_entry_update(
        &mut self,
        path: PathBuf,
        meta: Option<std::fs::Metadata>,
    ) -> Option<FileEntry> {
        let meta = meta.or_else(|| fs::metadata(&path).ok())?;

        // The following code in rust-analyzer is commented out
        // if meta.file_type().is_dir() && self
        //   .watched_entriesiter().any(|entry| entry.contains_dir(&path))
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

        let mut file = self.inner.content(&path).map(|it| (mtime, it));

        // Check state
        // Fast path: compare content
        match (&entry.prev, &mut file) {
            (None, ..) | (Some(Err(..)), Ok(..)) => {}
            (Some(..), Err(err)) => match &mut entry.state {
                WatchState::Fresh => {
                    if matches!(err, FileError::NotFound(..) | FileError::Other(..)) {
                        entry.state = WatchState::EmptyOrRemoval {
                            recheck_at: self.lifetime,
                            payload: file.clone(),
                        };
                        entry.prev = Some(file);
                        self.undetermined_send
                            .send(UndeterminedNotifyEvent {
                                at_realtime: instant::Instant::now(),
                                at_logical_tick: self.lifetime,
                                path: path.clone(),
                            })
                            .unwrap();
                        return None;
                    }
                }

                // Very complicated case of check error sequence, so we simplify
                // a bit, we regard any subsequent error as the same error.
                WatchState::EmptyOrRemoval { payload, .. } => {
                    // update payload
                    *payload = file;
                    return None;
                }
            },
            (Some(Ok((pt, prev))), Ok((nt, content))) => {
                // Both are Ok, so compare the content.
                if prev == content {
                    return None;
                }

                match entry.state {
                    WatchState::Fresh => {
                        if content.is_empty() {
                            entry.state = WatchState::EmptyOrRemoval {
                                recheck_at: self.lifetime,
                                payload: file.clone(),
                            };
                            entry.prev = Some(file);
                            self.undetermined_send
                                .send(UndeterminedNotifyEvent {
                                    at_realtime: instant::Instant::now(),
                                    at_logical_tick: self.lifetime,
                                    path,
                                })
                                .unwrap();
                            return None;
                        }
                    }

                    // Still empty
                    WatchState::EmptyOrRemoval { .. } if content.is_empty() => return None,
                    WatchState::EmptyOrRemoval { .. } => {
                        entry.state = WatchState::Fresh;
                    }
                }

                // this should be never happen, but we still check it
                if nt == pt {
                    // this is necessary to invalidate our mtime-based cache
                    *nt = pt.checked_add(std::time::Duration::from_micros(1)).unwrap();
                    log::warn!(
                        "same content but mtime is different...: {:?}",
                        entry.candidate_paths
                    );
                };
            }
        };

        entry.state = WatchState::Fresh;
        entry.prev = Some(file.clone());

        // Slow path: trigger the compiler
        Some((path, file.into()))
    }

    async fn recheck_notify_event(&mut self, event: UndeterminedNotifyEvent) -> Option<FileEntry> {
        let now = instant::Instant::now();
        log::info!("recheck event {event:?} at {now:?}");

        // The aysnc scheduler is not accurate, so we need to ensure a window here
        let reserved = now - event.at_realtime;
        if reserved < std::time::Duration::from_millis(50) {
            let send = self.undetermined_send.clone();
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(50) - reserved).await;
                send.send(event).unwrap();
            });
            return None;
        }

        let handle = same_file::Handle::from_path(&event.path).ok()?;

        let entry = self.watched_entries.get_mut(&handle)?;
        match std::mem::take(&mut entry.state) {
            WatchState::Fresh => None,
            WatchState::EmptyOrRemoval {
                recheck_at,
                payload,
            } => {
                if recheck_at == event.at_logical_tick {
                    log::info!("notify event is real {event:?}, state: {:?}", payload);
                    Some((event.path, payload.into()))
                } else {
                    None
                }
            }
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
