//! The [`CompileActor`] implementation borrowed from typst.ts.
//!
//! Please check `tinymist::actor::typ_client` for architecture details.

use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, OnceLock},
};

use reflexo::error::prelude::*;
use reflexo::typst::{TypstHtmlDocument, TypstPagedDocument};
use tinymist_world::{ConfigTask, OptionDocumentTask, ProjectInsId, WorldComputeGraph};
use tokio::sync::{mpsc, oneshot};

use crate::task::CacheTask;
use crate::vfs::notify::{FilesystemEvent, MemoryEvent, NotifyMessage, UpstreamUpdateEvent};
use crate::vfs::FsProvider;
use crate::world::{CompilerFeat, CompilerUniverse, EntryReader, RevisingUniverse, TaskInputs};
use crate::{watch::watch_deps, CompileSignal, TypstDocument};
use crate::{CompileReport, CompileSnapshot, WorldDeps};

pub trait CompilationHandle<F: CompilerFeat>: Send + Sync + 'static {
    fn status(&self, revision: usize, rep: CompileReport);
    // res: &CompiledArtifact<F>, rep: CompileReport
    fn notify_compile(&self, g: &Arc<WorldComputeGraph<F>>);
}

impl<F: CompilerFeat + Send + Sync + 'static> CompilationHandle<F>
    for std::marker::PhantomData<fn(F)>
{
    fn status(&self, _revision: usize, _: CompileReport) {}
    fn notify_compile(&self, _g: &Arc<WorldComputeGraph<F>>) {}
}

pub enum Interrupt<F: CompilerFeat> {
    /// Compile anyway.
    Compile,
    /// Compiled from computing thread.
    Compiled(Arc<WorldComputeGraph<F>>),
    /// Change the watching entry.
    ChangeTask(TaskInputs),
    /// Request compiler to respond a snapshot without needing to wait latest
    /// compilation.
    SnapshotRead(oneshot::Sender<CompileSnapshot<F>>),
    /// Request compiler to respond a snapshot with at least a compilation
    /// happens on or after current revision.
    CurrentRead(oneshot::Sender<Arc<WorldComputeGraph<F>>>),
    /// Memory file changes.
    Memory(MemoryEvent),
    /// File system event.
    Fs(FilesystemEvent),
    /// Request compiler to stop.
    Settle(oneshot::Sender<()>),
}

/// Responses from the compiler actor.
enum CompilerResponse {
    /// Response to the file watcher
    Notify(NotifyMessage),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct CompileReasons {
    /// The snapshot is taken by the memory editing events.
    by_memory_events: bool,
    /// The snapshot is taken by the file system events.
    by_fs_events: bool,
    /// The snapshot is taken by the entry change.
    by_entry_update: bool,
}

impl CompileReasons {
    fn see(&mut self, reason: CompileReasons) {
        self.by_memory_events |= reason.by_memory_events;
        self.by_fs_events |= reason.by_fs_events;
        self.by_entry_update |= reason.by_entry_update;
    }

    fn any(&self) -> bool {
        self.by_memory_events || self.by_fs_events || self.by_entry_update
    }
}

fn no_reason() -> CompileReasons {
    CompileReasons::default()
}

fn reason_by_mem() -> CompileReasons {
    CompileReasons {
        by_memory_events: true,
        ..CompileReasons::default()
    }
}

fn reason_by_fs() -> CompileReasons {
    CompileReasons {
        by_fs_events: true,
        ..CompileReasons::default()
    }
}

fn reason_by_entry_change() -> CompileReasons {
    CompileReasons {
        by_entry_update: true,
        ..CompileReasons::default()
    }
}

/// A tagged memory event with logical tick.
struct TaggedMemoryEvent {
    /// The logical tick when the event is received.
    logical_tick: usize,
    /// The memory event happened.
    event: MemoryEvent,
}

pub struct CompileServerOpts<F: CompilerFeat> {
    pub compile_handle: Arc<dyn CompilationHandle<F>>,
    pub cache: CacheTask,
}

impl<F: CompilerFeat + Send + Sync + 'static> Default for CompileServerOpts<F> {
    fn default() -> Self {
        Self {
            compile_handle: Arc::new(std::marker::PhantomData),
            cache: Default::default(),
        }
    }
}

/// The compiler actor.
pub struct CompileActor<F: CompilerFeat> {
    /// The underlying universe.
    pub verse: CompilerUniverse<F>,
    /// The compilation handle.
    pub compile_handle: Arc<dyn CompilationHandle<F>>,
    /// Whether to enable file system watching.
    pub enable_watch: bool,

    /// The current logical tick.
    logical_tick: usize,
    /// Last logical tick when invalidation is caused by shadow update.
    dirty_shadow_logical_tick: usize,

    /// Estimated latest set of shadow files.
    estimated_shadow_files: HashSet<Arc<Path>>,
    /// The latest compiled document.
    pub(crate) latest_doc: Option<Arc<WorldComputeGraph<F>>>,
    /// The latest successful document.
    pub(crate) latest_success_doc: Option<TypstDocument>,

    /// Channel for sending interrupts to the compiler actor.
    intr_tx: mpsc::UnboundedSender<Interrupt<F>>,
    /// Channel for receiving interrupts from the compiler actor.
    intr_rx: mpsc::UnboundedReceiver<Interrupt<F>>,
    /// Shared cache evict task.
    cache: CacheTask,

    watch_snap: OnceLock<CompileSnapshot<F>>,
    suspended: bool,
    compiling: bool,
    suspended_reason: CompileReasons,
    committed_revision: usize,
}

impl<F: CompilerFeat + Send + Sync + 'static> CompileActor<F> {
    /// Create a new compiler actor with options
    pub fn new_with(
        verse: CompilerUniverse<F>,
        intr_tx: mpsc::UnboundedSender<Interrupt<F>>,
        intr_rx: mpsc::UnboundedReceiver<Interrupt<F>>,
        CompileServerOpts {
            compile_handle,
            cache: cache_evict,
        }: CompileServerOpts<F>,
    ) -> Self {
        let entry = verse.entry_state();

        Self {
            verse,

            logical_tick: 1,
            compile_handle,
            enable_watch: false,
            dirty_shadow_logical_tick: 0,

            estimated_shadow_files: Default::default(),
            latest_doc: None,
            latest_success_doc: None,

            intr_tx,
            intr_rx,
            cache: cache_evict,

            watch_snap: OnceLock::new(),
            suspended: entry.is_inactive(),
            compiling: false,
            suspended_reason: no_reason(),
            committed_revision: 0,
        }
    }

    /// Create a new compiler actor.
    pub fn new(
        verse: CompilerUniverse<F>,
        intr_tx: mpsc::UnboundedSender<Interrupt<F>>,
        intr_rx: mpsc::UnboundedReceiver<Interrupt<F>>,
    ) -> Self {
        Self::new_with(verse, intr_tx, intr_rx, CompileServerOpts::default())
    }

    pub fn with_watch(mut self, watch: bool) -> Self {
        self.enable_watch = watch;
        self
    }

    /// Launches the compiler actor.
    pub async fn run(mut self) -> Result<bool> {
        if !self.enable_watch {
            // todo: once flag
            let g = self.compile_once().await;

            let report = g.get::<ConfigTask<CompileReport>>().transpose()?;
            let report = report.as_deref();
            let is_success = matches!(report, Some(CompileReport::CompileSuccess(..)));

            return Ok(is_success);
        }

        let (dep_tx, dep_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut curr_reads = vec![];

        log::debug!("CompileActor: initialized");

        // Trigger the first compilation (if active)
        self.run_compile(reason_by_entry_change(), &mut curr_reads, false);

        // Spawn file system watcher.
        let fs_tx = self.intr_tx.clone();
        tokio::spawn(watch_deps(dep_rx, move |event| {
            log_send_error("fs_event", fs_tx.send(Interrupt::Fs(event)));
        }));

        'event_loop: while let Some(mut event) = self.intr_rx.recv().await {
            let mut comp_reason = no_reason();

            'accumulate: loop {
                // Warp the logical clock by one.
                self.logical_tick += 1;

                // If settle, stop the actor.
                if let Interrupt::Settle(e) = event {
                    log::info!("CompileActor: requested stop");
                    e.send(()).ok();
                    break 'event_loop;
                }

                if let Interrupt::CurrentRead(event) = event {
                    curr_reads.push(event);
                } else {
                    comp_reason.see(self.process(event, |res: CompilerResponse| match res {
                        CompilerResponse::Notify(msg) => {
                            log_send_error("compile_deps", dep_tx.send(msg));
                        }
                    }));
                }

                // Try to accumulate more events.
                match self.intr_rx.try_recv() {
                    Ok(new_event) => event = new_event,
                    _ => break 'accumulate,
                }
            }

            // Either we have a reason to compile or we have events that want to have any
            // compilation.
            if comp_reason.any() || !curr_reads.is_empty() {
                self.run_compile(comp_reason, &mut curr_reads, false);
            }
        }

        log_send_error("settle_notify", dep_tx.send(NotifyMessage::Settle));
        log::info!("CompileActor: exited");
        Ok(true)
    }

    fn snapshot(&self, reason: CompileReasons) -> CompileSnapshot<F> {
        let world = self.verse.snapshot();
        CompileSnapshot {
            id: ProjectInsId::PRIMARY,
            world,
            signal: CompileSignal {
                by_entry_update: reason.by_entry_update,
                by_mem_events: reason.by_memory_events,
                by_fs_events: reason.by_fs_events,
            },
            success_doc: self.latest_success_doc.clone(),
        }
    }

    /// Compile the document once.
    pub async fn compile_once(&mut self) -> Arc<WorldComputeGraph<F>> {
        self.run_compile(reason_by_entry_change(), &mut vec![], true)
            .unwrap()
    }

    /// Compile the document once.
    fn run_compile(
        &mut self,
        reason: CompileReasons,
        curr_reads: &mut Vec<oneshot::Sender<Arc<WorldComputeGraph<F>>>>,
        is_once: bool,
    ) -> Option<Arc<WorldComputeGraph<F>>> {
        self.suspended_reason.see(reason);
        let reason = std::mem::take(&mut self.suspended_reason);
        let start = reflexo::time::now();

        let compiling = self.snapshot(reason);
        self.watch_snap = OnceLock::new();
        self.watch_snap.get_or_init(|| compiling.clone());

        if self.suspended {
            self.suspended_reason.see(reason);

            for reader in curr_reads.drain(..) {
                let _ = reader.send(WorldComputeGraph::new(compiling.clone()));
            }
            return None;
        }

        if self.compiling {
            self.suspended_reason.see(reason);
            return None;
        }

        self.compiling = true;

        let h = self.compile_handle.clone();

        // todo unwrap main id
        let id = compiling.world.main_id().unwrap();
        let revision = compiling.world.revision().get();

        h.status(revision, CompileReport::Stage(id, "compiling", start));

        let compile = move || {
            let compiling = WorldComputeGraph::new(compiling);

            h.notify_compile(&compiling);

            compiling
        };

        if is_once {
            Some(compile())
        } else {
            let intr_tx = self.intr_tx.clone();
            tokio::task::spawn_blocking(move || {
                log_send_error("compiled", intr_tx.send(Interrupt::Compiled(compile())));
            });

            None
        }
    }

    fn process_compile(
        &mut self,
        artifact: Arc<WorldComputeGraph<F>>,
        send: impl Fn(CompilerResponse),
    ) {
        self.compiling = false;

        let w = &artifact.snap.world;

        let compiled_revision = w.revision().get();
        if self.committed_revision >= compiled_revision {
            return;
        }

        let doc = {
            let paged = artifact
                .get::<OptionDocumentTask<TypstPagedDocument>>()
                .transpose()
                .ok()
                .flatten()
                .and_then(|e| e.as_ref().clone());
            let html = artifact
                .get::<OptionDocumentTask<TypstHtmlDocument>>()
                .transpose()
                .ok()
                .flatten()
                .and_then(|e| e.as_ref().clone());

            if let Some(paged) = paged {
                Some(TypstDocument::Paged(paged))
            } else {
                html.map(TypstDocument::Html)
            }
        };

        // Update state.
        self.committed_revision = compiled_revision;
        self.latest_doc = Some(artifact.clone());
        if doc.is_some() {
            self.latest_success_doc = doc;
        }

        // Notify the new file dependencies.
        let mut deps = vec![];
        w.iter_dependencies(&mut |dep| {
            if let Ok(x) = w.file_path(dep).and_then(|e| e.to_err()) {
                deps.push(x.into())
            }
        });
        send(CompilerResponse::Notify(NotifyMessage::SyncDependency(
            Box::new(deps),
        )));

        // Trigger an evict task.
        self.cache.evict();
    }

    /// Process some interrupt. Return whether it needs compilation.
    fn process(&mut self, event: Interrupt<F>, send: impl Fn(CompilerResponse)) -> CompileReasons {
        use CompilerResponse::*;

        match event {
            Interrupt::Compile => {
                // Increment the revision anyway.
                self.verse.increment_revision(|verse| {
                    verse.flush();
                });

                reason_by_entry_change()
            }
            Interrupt::SnapshotRead(task) => {
                log::debug!("CompileActor: take snapshot");
                if self
                    .watch_snap
                    .get()
                    .is_some_and(|e| e.world.revision() < self.verse.revision)
                {
                    self.watch_snap = OnceLock::new();
                }

                let _ = task.send(
                    self.watch_snap
                        .get_or_init(|| self.snapshot(no_reason()))
                        .clone(),
                );
                no_reason()
            }
            Interrupt::CurrentRead(..) => {
                unreachable!()
            }
            Interrupt::ChangeTask(change) => {
                self.verse.increment_revision(|verse| {
                    if let Some(inputs) = change.inputs {
                        verse.set_inputs(inputs);
                    }

                    if let Some(entry) = change.entry.clone() {
                        let res = verse.mutate_entry(entry);
                        if let Err(err) = res {
                            log::error!("CompileActor: change entry error: {err:?}");
                        }
                    }
                });

                // After incrementing the revision
                if let Some(entry) = change.entry {
                    self.suspended = entry.is_inactive();
                    if self.suspended {
                        log::info!("CompileActor: removing diag");
                        self.compile_handle
                            .status(self.verse.revision.get(), CompileReport::Suspend);
                    }

                    // Reset the watch state and document state.
                    self.latest_doc = None;
                    self.latest_success_doc = None;
                    self.suspended_reason = no_reason();
                }

                reason_by_entry_change()
            }
            Interrupt::Compiled(artifact) => {
                self.process_compile(artifact, send);
                self.process_lagged_compile()
            }
            Interrupt::Memory(event) => {
                log::debug!("CompileActor: memory event incoming");

                // Emulate memory changes.
                let mut files = HashSet::new();
                if matches!(event, MemoryEvent::Sync(..)) {
                    std::mem::swap(&mut files, &mut self.estimated_shadow_files);
                }

                let (MemoryEvent::Sync(e) | MemoryEvent::Update(e)) = &event;
                for path in &e.removes {
                    self.estimated_shadow_files.remove(path);
                    files.insert(Arc::clone(path));
                }
                for (path, _) in &e.inserts {
                    self.estimated_shadow_files.insert(Arc::clone(path));
                    files.remove(path);
                }

                // If there is no invalidation happening, apply memory changes directly.
                if files.is_empty() && self.dirty_shadow_logical_tick == 0 {
                    self.verse
                        .increment_revision(|verse| Self::apply_memory_changes(verse, event));
                    return reason_by_mem();
                }

                // Otherwise, send upstream update event.
                // Also, record the logical tick when shadow is dirty.
                self.dirty_shadow_logical_tick = self.logical_tick;
                send(Notify(NotifyMessage::UpstreamUpdate(UpstreamUpdateEvent {
                    invalidates: files.into_iter().collect(),
                    opaque: Box::new(TaggedMemoryEvent {
                        logical_tick: self.logical_tick,
                        event,
                    }),
                })));

                no_reason()
            }
            Interrupt::Fs(mut event) => {
                log::debug!("CompileActor: fs event incoming {event:?}");

                let mut reason = reason_by_fs();

                // Apply file system changes.
                let dirty_tick = &mut self.dirty_shadow_logical_tick;
                self.verse.increment_revision(|verse| {
                    // Handle delayed upstream update event before applying file system changes
                    if Self::apply_delayed_memory_changes(verse, dirty_tick, &mut event).is_none() {
                        log::warn!("CompileActor: unknown upstream update event");

                        // Actual a delayed memory event.
                        reason = reason_by_mem();
                    }
                    verse.vfs().notify_fs_event(event)
                });

                reason
            }
            Interrupt::Settle(_) => unreachable!(),
        }
    }

    /// Process reason after each compilation.
    fn process_lagged_compile(&mut self) -> CompileReasons {
        // The reason which is kept but not used.
        std::mem::take(&mut self.suspended_reason)
    }

    /// Apply delayed memory changes to underlying compiler.
    fn apply_delayed_memory_changes(
        verse: &mut RevisingUniverse<F>,
        dirty_shadow_logical_tick: &mut usize,
        event: &mut FilesystemEvent,
    ) -> Option<()> {
        // Handle delayed upstream update event before applying file system changes
        if let FilesystemEvent::UpstreamUpdate { upstream_event, .. } = event {
            let event = upstream_event.take()?.opaque;
            let TaggedMemoryEvent {
                logical_tick,
                event,
            } = *event.downcast().ok()?;

            // Recovery from dirty shadow state.
            if logical_tick == *dirty_shadow_logical_tick {
                *dirty_shadow_logical_tick = 0;
            }

            Self::apply_memory_changes(verse, event);
        }

        Some(())
    }

    /// Apply memory changes to underlying compiler.
    fn apply_memory_changes(verse: &mut RevisingUniverse<F>, event: MemoryEvent) {
        let mut vfs = verse.vfs();
        if matches!(event, MemoryEvent::Sync(..)) {
            vfs.reset_shadow();
        }
        match event {
            MemoryEvent::Update(event) | MemoryEvent::Sync(event) => {
                for path in event.removes {
                    let _ = vfs.unmap_shadow(&path);
                }
                for (path, snap) in event.inserts {
                    let _ = vfs.map_shadow(&path, snap);
                }
            }
        }
    }
}

#[inline]
fn log_send_error<T>(chan: &'static str, res: Result<(), mpsc::error::SendError<T>>) -> bool {
    res.map_err(|err| log::warn!("CompileActor: send to {chan} error: {err}"))
        .is_ok()
}
