//! upstream <https://github.com/rust-lang/rust-analyzer/tree/master/crates/vfs-notify>
//!
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
type NotifyFilePair = FileResult<(
    /* mtime */ instant::SystemTime,
    /* content */ Bytes,
)>;

/// The state of a watched file.
///
/// It is used to determine some dirty editiors' implementation.
#[derive(Debug)]
enum WatchState {
    /// The file is stable, which means we believe that it keeps syncthronized
    /// as expected.
    Stable,
    /// The file is empty or removed, but there is a chance that the file is not
    /// stable. So we need to recheck the file after a while.
    EmptyOrRemoval {
        recheck_at: usize,
        payload: NotifyFilePair,
    },
}

/// By default, the state is stable.
impl Default for WatchState {
    fn default() -> Self {
        Self::Stable
    }
}

/// The data entry of a watched file.
struct WatchedEntry {
    /// The lifetime of the entry.
    ///
    /// The entry will be removed if the entry is too old.
    // todo: generalize lifetime
    lifetime: usize,

    /// The state of the entry.
    state: WatchState,

    /// Previous content of the file.
    prev: Option<NotifyFilePair>,

    prev_meta: Option<std::fs::Metadata>,
}

/// Self produced event that check whether the file is stable after a while.
#[derive(Debug)]
struct UndeterminedNotifyEvent {
    /// The time when the event is produced.
    at_realtime: instant::Instant,
    /// The logical tick when the event is produced.
    at_logical_tick: usize,
    /// The path of the file.
    path: PathBuf,
}

// Drop order is significant.
/// The actor that watches files.
/// It is used to watch files and send events to the consumers
pub struct NotifyActor {
    /// The access model of the actor.
    /// We concrete the access model to `SystemAccessModel` for now.
    inner: SystemAccessModel,

    /// The lifetime tick of the actor.
    lifetime: usize,

    /// The logical tick of the actor.
    logical_tick: usize,

    /// Whether the actor is using builtin watcher.
    ///
    /// If it is [`None`], the actor need a upstream event to trigger the
    /// updates
    watch: Option<()>,

    /// Output of the actor.
    /// See [`FilesystemEvent`] for more information.
    sender: mpsc::UnboundedSender<FilesystemEvent>,

    /// Internal channel for recheck events.
    undetermined_send: mpsc::UnboundedSender<UndeterminedNotifyEvent>,
    undetermined_recv: mpsc::UnboundedReceiver<UndeterminedNotifyEvent>,

    /// The holded entries for watching, one entry for per file.
    watched_entries: HashMap<PathBuf, WatchedEntry>,

    /// The builtin watcher object.
    watcher: Option<WatcherPair>,
}

impl NotifyActor {
    /// Create a new actor.
    fn new(sender: mpsc::UnboundedSender<FilesystemEvent>) -> NotifyActor {
        let (undetermined_send, undetermined_recv) = mpsc::unbounded_channel();

        NotifyActor {
            inner: SystemAccessModel,
            // we start from 1 to distinguish from 0 (default value)
            lifetime: 1,
            logical_tick: 1,

            watch: Some(()),
            sender,

            undetermined_send,
            undetermined_recv,

            watched_entries: HashMap::new(),
            watcher: None,
        }
    }

    /// Send a message to the actor.
    fn send(&mut self, msg: FilesystemEvent) {
        self.sender.send(msg).unwrap();
    }

    /// Get the notify event from the watcher.
    async fn get_notify_event(watcher: &mut Option<WatcherPair>) -> Option<NotifyEvent> {
        match watcher {
            Some((_, watcher_receiver)) => watcher_receiver.recv().await,
            None => None,
        }
    }

    /// Main loop of the actor.
    async fn run(mut self, mut inbox: mpsc::UnboundedReceiver<NotifyMessage>) {
        /// The event of the actor.
        #[derive(Debug)]
        enum ActorEvent {
            /// Recheck the notify event.
            ReCheck(UndeterminedNotifyEvent),
            /// external message to change notifer's state
            Message(NotifyMessage),
            /// notify event from builtin watcher
            NotifyEvent(NotifyEvent),
        }

        loop {
            // Get the event from the inbox or the watcher.
            let event = tokio::select! {
                Some(it) = inbox.recv() => Some(ActorEvent::Message(it)),
                Some(it) = Self::get_notify_event(&mut self.watcher) => Some(ActorEvent::NotifyEvent(it)),
                Some(it) = self.undetermined_recv.recv() => Some(ActorEvent::ReCheck(it)),
            };

            // Failed to get the event.
            let Some(event) = event else {
                log::info!("failed to get event, exiting...");
                return;
            };

            // Increase the logical tick per event.
            self.logical_tick += 1;

            // log::info!("vfs-notify event {event:?}");
            // function entries to handle some event
            match event {
                ActorEvent::Message(NotifyMessage::SyncDependency(paths)) => {
                    self.update_watches(paths).await;
                }
                ActorEvent::NotifyEvent(event) => {
                    // log::info!("notify event {event:?}");
                    if let Some(event) = log_notify_error(event, "failed to notify") {
                        self.notify_event(event).await;
                    }
                }
                ActorEvent::ReCheck(event) => {
                    self.recheck_notify_event(event).await;
                }
            }
        }
    }

    /// Update the watches of corresponding files.
    async fn update_watches(&mut self, paths: Vec<PathBuf>) {
        // Increase the lifetime per external message.
        self.lifetime += 1;

        let mut changeset = FileChangeSet::default();

        // Remove old watches, if any.
        self.watcher = None;
        if self.watch.is_some() {
            match &mut self.watcher {
                // Clear the old watches.
                Some((old_watcher, _)) => {
                    for path in self.watched_entries.keys() {
                        // Remove the watch if it still exists.
                        if let Err(err) = old_watcher.unwatch(path) {
                            if !matches!(err.kind, notify::ErrorKind::WatchNotFound) {
                                log::warn!("failed to unwatch: {err}");
                            }
                        }
                    }
                }
                // Create a new builtin watcher.
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
        //
        // Also check whether the file is updated since there is a window
        // between unwatch the file and watch the file again.
        for path in paths.into_iter() {
            // Update or insert the entry with the new lifetime.
            let entry = self
                .watched_entries
                .entry(path.clone())
                .and_modify(|watch_entry| {
                    watch_entry.lifetime = self.lifetime;
                })
                .or_insert_with(|| WatchedEntry {
                    lifetime: self.lifetime,
                    state: WatchState::Stable,
                    prev: None,
                    prev_meta: None,
                });

            // Update in-memory metadata for now.
            let Some(meta) = path.metadata().ok().or(entry.prev_meta.clone()) else {
                // We cannot get the metadata even at the first time, so we are
                // okay to ignore this file for watching.
                continue;
            };
            entry.prev_meta = Some(meta.clone());

            let watch = self.watch.is_some();
            if watch {
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

        // Remove old entries.
        // Note: since we have increased the lifetime, it is safe to remove the
        // old entries after updating the watched entries.
        self.watched_entries.retain(|path, entry| {
            if self.lifetime - entry.lifetime < 30 {
                true
            } else {
                changeset.removes.push(path.clone());
                false
            }
        });

        if !changeset.is_empty() {
            self.send(FilesystemEvent::Update(changeset));
        }
    }

    /// Notify the event from the builtin watcher.
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

    /// Notify any update of the file entry
    fn notify_entry_update(
        &mut self,
        path: PathBuf,
        meta: Option<std::fs::Metadata>,
    ) -> Option<FileEntry> {
        let meta = meta.or_else(|| fs::metadata(&path).ok())?;

        // The following code in rust-analyzer is commented out
        // todo: check whether we need this
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

        // Find entry and continue
        let entry = self.watched_entries.get_mut(&path)?;

        let mut file = self.inner.content(&path).map(|it| (mtime, it));

        // Check state in fast path: compare state, return None on not sending
        // the file change
        match (&entry.prev, &mut file) {
            // update the content of the entry in the following cases:
            // + Case 1: previous content is clear
            // + Case 2: previous content is not clear but some error, and the
            // current content is ok
            (None, ..) | (Some(Err(..)), Ok(..)) => {}
            // Meet some error currently
            (Some(..), Err(err)) => match &mut entry.state {
                // If the file is stable, check whether the editor is removing
                // or truncating the file. They are possibly flushing the file
                // but not finished yet.
                WatchState::Stable => {
                    if matches!(err, FileError::NotFound(..) | FileError::Other(..)) {
                        entry.state = WatchState::EmptyOrRemoval {
                            recheck_at: self.logical_tick,
                            payload: file.clone(),
                        };
                        entry.prev = Some(file);
                        self.undetermined_send
                            .send(UndeterminedNotifyEvent {
                                at_realtime: instant::Instant::now(),
                                at_logical_tick: self.logical_tick,
                                path: path.clone(),
                            })
                            .unwrap();
                        return None;
                    }
                    // Otherwise, we push the error to the consumer.
                }

                // Very complicated case of check error sequence, so we simplify
                // a bit, we regard any subsequent error as the same error.
                WatchState::EmptyOrRemoval { payload, .. } => {
                    // update payload
                    *payload = file;
                    return None;
                }
            },
            // Compare content for transitinal the state
            (Some(Ok((prev_tick, prev_content))), Ok((next_tick, next_content))) => {
                // So far it is acurately no change for the file, skip it
                if prev_content == next_content {
                    return None;
                }

                match entry.state {
                    // If the file is stable, check whether the editor is
                    // removing or truncating the file. They are possibly
                    // flushing the file but not finished yet.
                    WatchState::Stable => {
                        if next_content.is_empty() {
                            entry.state = WatchState::EmptyOrRemoval {
                                recheck_at: self.logical_tick,
                                payload: file.clone(),
                            };
                            entry.prev = Some(file);
                            self.undetermined_send
                                .send(UndeterminedNotifyEvent {
                                    at_realtime: instant::Instant::now(),
                                    at_logical_tick: self.logical_tick,
                                    path,
                                })
                                .unwrap();
                            return None;
                        }
                    }

                    // Still empty
                    WatchState::EmptyOrRemoval { .. } if next_content.is_empty() => return None,
                    // Otherwise, we push the diff to the consumer.
                    WatchState::EmptyOrRemoval { .. } => {}
                }

                // We have found a change, however, we need to check whether the
                // mtime is changed. Generally, the mtime should be changed.
                // However, It is common that editor (VSCode) to change the
                // mtime after writing
                //
                // this condition should be never happen, but we still check it
                //
                // There will be cases that user change content of a file and
                // then also modify the mtime of the file, so we need to check
                // `next_tick == prev_tick`: Whether mtime is changed.
                // `matches!(entry.state, WatchState::Fresh)`: Whether the file
                //   is fresh. We have not submit the file to the compiler, so
                //   that is ok.
                if next_tick == prev_tick && matches!(entry.state, WatchState::Stable) {
                    // this is necessary to invalidate our mtime-based cache
                    *next_tick = prev_tick
                        .checked_add(std::time::Duration::from_micros(1))
                        .unwrap();
                    log::warn!("same content but mtime is different...: {:?} content: prev:{:?} v.s. curr:{:?}", path, prev_content, next_content);
                };
            }
        };

        // Send the update to the consumer
        // Update the entry according to the state
        entry.state = WatchState::Stable;
        entry.prev = Some(file.clone());

        // Slow path: trigger the file change for consumer
        Some((path, file.into()))
    }

    /// Recheck the notify event after a while.
    async fn recheck_notify_event(&mut self, event: UndeterminedNotifyEvent) -> Option<()> {
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

        // Check whether the entry is still valid
        let entry = self.watched_entries.get_mut(&event.path)?;

        // Check the state of the entry
        match std::mem::take(&mut entry.state) {
            // If the entry is stable, we do nothing
            WatchState::Stable => {}
            // If the entry is not stable, and no other event is produced after
            // this event, we send the event to the consumer.
            WatchState::EmptyOrRemoval {
                recheck_at,
                payload,
            } => {
                if recheck_at == event.at_logical_tick {
                    log::info!("notify event is real {event:?}, state: {:?}", payload);

                    // Send the underlying change to the consumer
                    let mut changeset = FileChangeSet::default();
                    changeset.inserts.push((event.path, payload.into()));
                    self.send(FilesystemEvent::Update(changeset));
                }
            }
        };

        Some(())
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
