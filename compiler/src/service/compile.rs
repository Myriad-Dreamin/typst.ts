use std::{
    collections::HashSet,
    num::NonZeroUsize,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
    thread::JoinHandle,
};

use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use typst::{
    layout::{Frame, FrameItem, Point, Position},
    syntax::{LinkedNode, Source, Span, SyntaxKind, VirtualPath},
    World,
};

use crate::{
    service::features::WITH_COMPILING_STATUS_FEATURE,
    vfs::notify::{FilesystemEvent, MemoryEvent, NotifyMessage},
    world::{CompilerFeat, CompilerWorld},
    ShadowApi,
};
use typst_ts_core::{
    error::prelude::{map_string_err, ZResult},
    TypstDocument, TypstFileId,
};

use super::{
    features::FeatureSet, CompileEnv, CompileReporter, Compiler, ConsoleDiagReporter,
    WorkspaceProvider, WorldExporter,
};

/// A task that can be sent to the context (compiler thread)
///
/// The internal function will be dereferenced and called on the context.
type BorrowTask<Ctx> = Box<dyn FnOnce(&mut Ctx) + Send + 'static>;

/// Interrupts for the compiler thread.
enum CompilerInterrupt<Ctx> {
    /// Interrupted by task.
    ///
    /// See [`CompileClient<Ctx>::steal`] for more information.
    Task(BorrowTask<Ctx>),
    /// Interrupted by memory file changes.
    Memory(MemoryEvent),
    /// Interrupted by file system event.
    ///
    /// If the event is `None`, it means the initial file system scan is done.
    /// Otherwise, it means a file system event is received.
    Fs(Option<FilesystemEvent>),
}

/// Responses from the compiler thread.
enum CompilerResponse {
    /// Response to the file watcher
    Notify(NotifyMessage),
}

/// A tagged memory event with logical tick.
struct TaggedMemoryEvent {
    /// The logical tick when the event is received.
    logical_tick: usize,
    /// The memory event happened.
    event: MemoryEvent,
}

/// The compiler thread.
pub struct CompileActor<C: Compiler> {
    /// The underlying compiler.
    pub compiler: CompileReporter<C>,
    /// The root path of the workspace.
    pub root: PathBuf,
    /// Whether to enable file system watching.
    pub enable_watch: bool,

    /// The current logical tick.
    logical_tick: usize,
    /// Last logical tick when invalidation is caused by shadow update.
    dirty_shadow_logical_tick: usize,

    /// Estimated latest set of shadow files.
    estimated_shadow_files: HashSet<Arc<Path>>,
    /// The latest compiled document.
    latest_doc: Option<Arc<TypstDocument>>,
    /// feature set for compile_once mode.
    once_feature_set: FeatureSet,
    /// Shared feature set for watch mode.
    watch_feature_set: Arc<FeatureSet>,

    /// Internal channel for stealing the compiler thread.
    steal_send: mpsc::UnboundedSender<BorrowTask<Self>>,
    steal_recv: mpsc::UnboundedReceiver<BorrowTask<Self>>,

    /// Internal channel for memory events.
    memory_send: mpsc::UnboundedSender<MemoryEvent>,
    memory_recv: mpsc::UnboundedReceiver<MemoryEvent>,
}

impl<C: Compiler + ShadowApi + WorldExporter + Send + 'static> CompileActor<C>
where
    C::World: for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    pub fn new_with_features(compiler: C, root: PathBuf, feature_set: FeatureSet) -> Self {
        let (steal_send, steal_recv) = mpsc::unbounded_channel();
        let (memory_send, memory_recv) = mpsc::unbounded_channel();

        let watch_feature_set = Arc::new(
            feature_set
                .clone()
                .configure(&WITH_COMPILING_STATUS_FEATURE, true),
        );

        Self {
            compiler: CompileReporter::new(compiler)
                .with_generic_reporter(ConsoleDiagReporter::default()),
            root,

            logical_tick: 1,
            enable_watch: false,
            dirty_shadow_logical_tick: 0,

            estimated_shadow_files: Default::default(),
            latest_doc: None,
            once_feature_set: feature_set,
            watch_feature_set,

            steal_send,
            steal_recv,

            memory_send,
            memory_recv,
        }
    }

    /// Create a new compiler thread.
    pub fn new(compiler: C, root: PathBuf) -> Self {
        Self::new_with_features(compiler, root, FeatureSet::default())
    }

    /// Run the compiler thread synchronously.
    pub fn run(self) -> bool {
        use tokio::runtime::Handle;

        if Handle::try_current().is_err() && self.enable_watch {
            log::error!("Typst compiler thread with watch enabled must be run in a tokio runtime");
            return false;
        }

        tokio::task::block_in_place(move || Handle::current().block_on(self.block_run_inner()))
    }

    /// Inner function for `run`, it launches the compiler thread and blocks
    /// until it exits.
    async fn block_run_inner(mut self) -> bool {
        if !self.enable_watch {
            let compiled = self
                .compiler
                .compile(&mut CompileEnv::default().configure(self.once_feature_set));
            return compiled.is_ok();
        }

        if let Some(h) = self.spawn().await {
            // Note: this is blocking the current thread.
            // Note: the block safety is ensured by `run` function.
            h.join().unwrap();
        }

        true
    }

    /// Spawn the compiler thread.
    pub async fn spawn(mut self) -> Option<JoinHandle<()>> {
        if !self.enable_watch {
            self.compiler
                .compile(&mut CompileEnv::default().configure(self.once_feature_set))
                .ok();
            return None;
        }

        // Setup internal channels.
        let (dep_tx, dep_rx) = tokio::sync::mpsc::unbounded_channel();
        let (fs_tx, mut fs_rx) = tokio::sync::mpsc::unbounded_channel();

        // Wrap sender to send compiler response.
        let compiler_ack = move |res: CompilerResponse| match res {
            CompilerResponse::Notify(msg) => {
                log_send_error("compile_deps", dep_tx.send(msg));
            }
        };

        // Spawn file system watcher.
        tokio::spawn(super::watch_deps(dep_rx, move |event| {
            log_send_error("fs_event", fs_tx.send(event));
        }));

        // Spawn compiler thread.
        let compile_thread = ensure_single_thread("typst-compiler", async move {
            log::debug!("CompileActor: initialized");

            // Wait for first events.
            while let Some(event) = tokio::select! {
                Some(it) = fs_rx.recv() => Some(CompilerInterrupt::Fs(it)),
                Some(it) = self.memory_recv.recv() => Some(CompilerInterrupt::Memory(it)),
                Some(it) = self.steal_recv.recv() => Some(CompilerInterrupt::Task(it)),
            } {
                // Small step to warp the logical clock.
                self.logical_tick += 1;

                // Accumulate events.
                let mut need_recompile = false;
                need_recompile = self.process(event, &compiler_ack) || need_recompile;
                while let Some(event) = fs_rx
                    .try_recv()
                    .ok()
                    .map(CompilerInterrupt::Fs)
                    .or_else(|| {
                        self.memory_recv
                            .try_recv()
                            .ok()
                            .map(CompilerInterrupt::Memory)
                    })
                    .or_else(|| self.steal_recv.try_recv().ok().map(CompilerInterrupt::Task))
                {
                    need_recompile = self.process(event, &compiler_ack) || need_recompile;
                }

                // Compile if needed.
                if need_recompile {
                    self.compile(&compiler_ack);
                }
            }

            log::debug!("CompileActor: exited");
        })
        .unwrap();

        // Return the thread handle.
        Some(compile_thread)
    }

    /// Compile the document.
    fn compile(&mut self, send: impl Fn(CompilerResponse)) {
        use CompilerResponse::*;

        // Compile the document.
        self.latest_doc = self
            .compiler
            .compile(&mut CompileEnv::default().configure_shared(self.watch_feature_set.clone()))
            .ok();

        // Evict compilation cache.
        comemo::evict(30);

        // Notify the new file dependencies.
        let mut deps = vec![];
        self.compiler
            .iter_dependencies(&mut |dep, _| deps.push(dep.clone()));
        send(Notify(NotifyMessage::SyncDependency(deps)));
    }

    /// Process some interrupt.
    fn process(&mut self, event: CompilerInterrupt<Self>, send: impl Fn(CompilerResponse)) -> bool {
        use CompilerResponse::*;
        // warp the logical clock by one.
        self.logical_tick += 1;

        match event {
            // Borrow the compiler thread and run the task.
            //
            // See [`CompileClient::steal`] for more information.
            CompilerInterrupt::Task(task) => {
                log::debug!("CompileActor: execute task");

                task(self);

                // Will never trigger compilation
                false
            }
            // Handle memory events.
            CompilerInterrupt::Memory(event) => {
                log::debug!("CompileActor: memory event incoming");

                // Emulate memory changes.
                let mut files = HashSet::new();
                if matches!(event, MemoryEvent::Sync(..)) {
                    files = self.estimated_shadow_files.clone();
                    self.estimated_shadow_files.clear();
                }
                match &event {
                    MemoryEvent::Sync(event) | MemoryEvent::Update(event) => {
                        for path in event.removes.iter().map(Deref::deref) {
                            self.estimated_shadow_files.remove(path);
                            files.insert(path.into());
                        }
                        for path in event.inserts.iter().map(|e| e.0.deref()) {
                            self.estimated_shadow_files.insert(path.into());
                            files.remove(path);
                        }
                    }
                }

                // If there is no invalidation happening, apply memory changes directly.
                if files.is_empty() && self.dirty_shadow_logical_tick == 0 {
                    self.apply_memory_changes(event);

                    // Will trigger compilation
                    return true;
                }

                // Otherwise, send upstream update event.
                // Also, record the logical tick when shadow is dirty.
                self.dirty_shadow_logical_tick = self.logical_tick;
                send(Notify(NotifyMessage::UpstreamUpdate(
                    crate::vfs::notify::UpstreamUpdateEvent {
                        invalidates: files.into_iter().collect(),
                        opaque: Box::new(TaggedMemoryEvent {
                            logical_tick: self.logical_tick,
                            event,
                        }),
                    },
                )));

                // Delayed trigger compilation
                false
            }
            // Handle file system events.
            CompilerInterrupt::Fs(event) => {
                log::debug!("CompileActor: fs event incoming {:?}", event);

                // Handle file system event if any.
                if let Some(mut event) = event {
                    // Handle delayed upstream update event before applying file system changes
                    if self.apply_delayed_memory_changes(&mut event).is_none() {
                        log::warn!("CompileActor: unknown upstream update event");
                    }

                    // Apply file system changes.
                    self.compiler.notify_fs_event(event);
                }

                // Will trigger compilation
                true
            }
        }
    }

    /// Apply delayed memory changes to underlying compiler.
    fn apply_delayed_memory_changes(&mut self, event: &mut FilesystemEvent) -> Option<()> {
        // Handle delayed upstream update event before applying file system changes
        if let FilesystemEvent::UpstreamUpdate { upstream_event, .. } = event {
            let event = upstream_event.take()?.opaque;
            let TaggedMemoryEvent {
                logical_tick,
                event,
            } = *event.downcast().ok()?;

            // Recovery from dirty shadow state.
            if logical_tick == self.dirty_shadow_logical_tick {
                self.dirty_shadow_logical_tick = 0;
            }

            self.apply_memory_changes(event);
        }

        Some(())
    }

    /// Apply memory changes to underlying compiler.
    fn apply_memory_changes(&mut self, event: MemoryEvent) {
        if matches!(event, MemoryEvent::Sync(..)) {
            self.compiler.reset_shadow();
        }
        match event {
            MemoryEvent::Update(event) | MemoryEvent::Sync(event) => {
                for removes in event.removes {
                    let _ = self.compiler.unmap_shadow(&removes);
                }
                for (p, t) in event.inserts {
                    let insert_file = match t.content().cloned() {
                        Ok(content) => content,
                        Err(err) => {
                            log::error!(
                                "CompileActor: read memory file at {}: {}",
                                p.display(),
                                err,
                            );
                            continue;
                        }
                    };

                    let _ = self.compiler.map_shadow(&p, insert_file);
                }
            }
        }
    }
}

impl<C: Compiler> CompileActor<C> {
    pub fn with_watch(mut self, enable_watch: bool) -> Self {
        self.enable_watch = enable_watch;
        self
    }

    pub fn split(self) -> (Self, CompileClient<Self>) {
        let steal_send = self.steal_send.clone();
        let memory_send = self.memory_send.clone();
        (
            self,
            CompileClient {
                steal_send,
                memory_send,
                _ctx: std::marker::PhantomData,
            },
        )
    }

    pub fn document(&self) -> Option<Arc<TypstDocument>> {
        self.latest_doc.clone()
    }
}
#[derive(Debug, Clone)]
pub struct CompileClient<Ctx> {
    steal_send: mpsc::UnboundedSender<BorrowTask<Ctx>>,
    memory_send: mpsc::UnboundedSender<MemoryEvent>,

    _ctx: std::marker::PhantomData<Ctx>,
}

impl<Ctx> CompileClient<Ctx> {
    fn steal_inner<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx) -> Ret + Send + 'static,
    ) -> ZResult<oneshot::Receiver<Ret>> {
        let (tx, rx) = oneshot::channel();

        let task = Box::new(move |this: &mut Ctx| {
            if tx.send(f(this)).is_err() {
                // Receiver was dropped. The main thread may have exited, or the request may
                // have been cancelled.
                log::warn!("could not send back return value from Typst thread");
            }
        });

        self.steal_send
            .send(task)
            .map_err(map_string_err("failed to send to steal"))?;
        Ok(rx)
    }

    pub fn steal<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx) -> Ret + Send + 'static,
    ) -> ZResult<Ret> {
        self.steal_inner(f)?
            .blocking_recv()
            .map_err(map_string_err("failed to recv from steal"))
    }

    /// Steal the compiler thread and run the given function.
    pub async fn steal_async<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx, tokio::runtime::Handle) -> Ret + Send + 'static,
    ) -> ZResult<Ret> {
        // get current async handle
        let handle = tokio::runtime::Handle::current();
        self.steal_inner(move |this: &mut Ctx| f(this, handle.clone()))?
            .await
            .map_err(map_string_err("failed to call steal_async"))
    }

    pub fn add_memory_changes(&self, event: MemoryEvent) {
        log_send_error("mem_event", self.memory_send.send(event));
    }
}

#[derive(Debug, Serialize)]
pub struct DocToSrcJumpInfo {
    pub filepath: String,
    pub start: Option<(usize, usize)>, // row, column
    pub end: Option<(usize, usize)>,
}

// todo: remove constraint to CompilerWorld
impl<F: CompilerFeat, Ctx: Compiler<World = CompilerWorld<F>>> CompileClient<CompileActor<Ctx>>
where
    Ctx::World: WorkspaceProvider,
{
    /// fixme: character is 0-based, UTF-16 code unit.
    /// We treat it as UTF-8 now.
    pub async fn resolve_src_to_doc_jump(
        &mut self,
        filepath: PathBuf,
        line: usize,
        character: usize,
    ) -> ZResult<Option<Position>> {
        self.steal_async(move |this, _| {
            let doc = this.document()?;

            let world = this.compiler.world();

            let relative_path = filepath
                .strip_prefix(&this.compiler.world().workspace_root())
                .ok()?;

            let source_id = TypstFileId::new(None, VirtualPath::new(relative_path));
            let source = world.source(source_id).ok()?;
            let cursor = source.line_column_to_byte(line, character)?;

            jump_from_cursor(&doc.pages, &source, cursor)
        })
        .await
    }

    pub async fn resolve_span(&mut self, span: Span) -> ZResult<Option<DocToSrcJumpInfo>> {
        self.resolve_span_and_offset(span, None).await
    }

    pub async fn resolve_span_and_offset(
        &mut self,
        span: Span,
        offset: Option<usize>,
    ) -> ZResult<Option<DocToSrcJumpInfo>> {
        let resolve_off =
            |src: &Source, off: usize| src.byte_to_line(off).zip(src.byte_to_column(off));

        self.steal_async(move |this, _| {
            let world = this.compiler.world();
            let src_id = span.id()?;
            let source = world.source(src_id).ok()?;
            let mut range = source.find(span)?.range();
            if let Some(off) = offset {
                if off < range.len() {
                    range.start += off;
                }
            }
            let filepath = world.path_for_id(src_id).ok()?;
            Some(DocToSrcJumpInfo {
                filepath: filepath.to_string_lossy().to_string(),
                start: resolve_off(&source, range.start),
                end: resolve_off(&source, range.end),
            })
        })
        .await
    }
}

/// Spawn a thread and run the given future on it.
///
/// Note: the future is run on a single-threaded tokio runtime.
fn ensure_single_thread<F: std::future::Future<Output = ()> + Send + 'static>(
    name: &str,
    f: F,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new().name(name.to_owned()).spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f);
    })
}

/// Find the output location in the document for a cursor position.
pub fn jump_from_cursor(frames: &[Frame], source: &Source, cursor: usize) -> Option<Position> {
    let node = LinkedNode::new(source.root()).leaf_at(cursor)?;
    if node.kind() != SyntaxKind::Text {
        return None;
    }

    let mut min_dis = u64::MAX;
    let mut p = Point::default();
    let mut ppage = 0usize;

    let span = node.span();
    for (i, frame) in frames.iter().enumerate() {
        let t_dis = min_dis;
        if let Some(pos) = find_in_frame(frame, span, &mut min_dis, &mut p) {
            return Some(Position {
                page: NonZeroUsize::new(i + 1)?,
                point: pos,
            });
        }
        if t_dis != min_dis {
            ppage = i;
        }
    }

    if min_dis == u64::MAX {
        return None;
    }

    Some(Position {
        page: NonZeroUsize::new(ppage + 1)?,
        point: p,
    })
}

/// Find the position of a span in a frame.
fn find_in_frame(frame: &Frame, span: Span, min_dis: &mut u64, p: &mut Point) -> Option<Point> {
    for (mut pos, item) in frame.items() {
        if let FrameItem::Group(group) = item {
            // TODO: Handle transformation.
            if let Some(point) = find_in_frame(&group.frame, span, min_dis, p) {
                return Some(point + pos);
            }
        }

        if let FrameItem::Text(text) = item {
            for glyph in &text.glyphs {
                if glyph.span.0 == span {
                    return Some(pos);
                }
                if glyph.span.0.id() == span.id() {
                    let dis = glyph.span.0.number().abs_diff(span.number());
                    if dis < *min_dis {
                        *min_dis = dis;
                        *p = pos;
                    }
                }
                pos.x += glyph.x_advance.at(text.size);
            }
        }
    }

    None
}

#[inline]
fn log_send_error<T>(chan: &'static str, res: Result<(), mpsc::error::SendError<T>>) -> bool {
    res.map_err(|err| log::warn!("CompileActor: send to {chan} error: {err}"))
        .is_ok()
}
