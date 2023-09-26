use std::{num::NonZeroUsize, path::PathBuf, sync::Arc, thread::JoinHandle};

use serde::Serialize;
use tokio::sync::{mpsc, oneshot};
use typst::{
    doc::{Frame, FrameItem, Position},
    geom::Point,
    syntax::{LinkedNode, Source, Span, SyntaxKind, VirtualPath},
    World,
};

use crate::{
    vfs::notify::{FileChangeSet, FilesystemEvent, NotifyMessage},
    world::{CompilerFeat, CompilerWorld},
    ShadowApi,
};
use typst_ts_core::{
    error::prelude::ZResult, vector::span_id_from_u64, TypstDocument, TypstFileId,
};

use super::{Compiler, WorkspaceProvider, WorldExporter};

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

pub enum MemoryEvent {
    Sync(FileChangeSet),
    Update(FileChangeSet),
}

enum CompilerInterrupt<Ctx> {
    /// Interrupted by file system event.
    Fs(Option<FilesystemEvent>),
    /// Interrupted by memory file changes.
    Memory(MemoryEvent),
    /// Interrupted by task.
    Task(BorrowTask<Ctx>),
}

enum CompilerResponse {
    Notify(NotifyMessage),
}

type BorrowTask<Ctx> = Box<dyn FnOnce(&mut Ctx) + Send + 'static>;

pub struct CompileActor<C: Compiler> {
    pub compiler: C,
    pub root: PathBuf,
    pub enable_watch: bool,

    latest_doc: Option<Arc<TypstDocument>>,

    steal_send: mpsc::UnboundedSender<BorrowTask<Self>>,
    steal_recv: mpsc::UnboundedReceiver<BorrowTask<Self>>,

    memory_send: mpsc::UnboundedSender<MemoryEvent>,
    memory_recv: mpsc::UnboundedReceiver<MemoryEvent>,
}

// todo: remove cfg feature here
#[cfg(feature = "system-watch")]
use super::DiagObserver;
#[cfg(feature = "system-watch")]
impl<C: Compiler + ShadowApi + WorldExporter + Send + 'static> CompileActor<C>
where
    C::World: for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    pub fn new(compiler: C, root: PathBuf) -> Self {
        let (steal_send, steal_recv) = mpsc::unbounded_channel();
        let (memory_send, memory_recv) = mpsc::unbounded_channel();

        Self {
            compiler,
            root,
            enable_watch: false,

            latest_doc: None,

            steal_send,
            steal_recv,

            memory_send,
            memory_recv,
        }
    }

    fn compile(&mut self, send: impl Fn(CompilerResponse)) {
        use CompilerResponse::*;

        // compile
        self.compiler
            .with_stage_diag::<true, _>("compiling", |driver| driver.compile());

        comemo::evict(30);

        let mut deps = vec![];
        self.compiler
            .iter_dependencies(&mut |dep, _| deps.push(dep.to_owned()));
        send(Notify(NotifyMessage::SyncDependency(deps)));
        // tx
    }

    fn process(&mut self, event: CompilerInterrupt<Self>) -> bool {
        match event {
            CompilerInterrupt::Fs(event) => {
                log::info!("CompileActor: fs event incoming {:?}", event);

                if let Some(event) = event {
                    self.compiler.notify_fs_event(event);
                }

                true
            }
            CompilerInterrupt::Memory(event) => {
                log::info!("CompileActor: memory event incoming");

                if matches!(event, MemoryEvent::Sync(..)) {
                    self.compiler.reset_shadow();
                }
                match event {
                    MemoryEvent::Update(event) | MemoryEvent::Sync(event) => {
                        for removes in event.removes {
                            let _ = self.compiler.unmap_shadow(&removes);
                        }
                        for (p, t) in event.inserts {
                            let _ = self.compiler.map_shadow(&p, t.content().cloned().unwrap());
                        }
                    }
                }

                true
            }
            CompilerInterrupt::Task(task) => {
                task(self);
                false
            }
        }
    }

    pub async fn spawn(mut self) -> Option<JoinHandle<()>> {
        if !self.enable_watch {
            self.compiler
                .with_stage_diag::<false, _>("compiling", |driver| driver.compile());
            return None;
        }

        let (dep_tx, dep_rx) = tokio::sync::mpsc::unbounded_channel();
        let (fs_tx, mut fs_rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(super::watch_deps(dep_rx, move |event| {
            fs_tx.send(event).unwrap();
        }));

        let compiler_ack = move |res: CompilerResponse| match res {
            CompilerResponse::Notify(msg) => dep_tx.send(msg).unwrap(),
        };

        let compile_thread = ensure_single_thread("typst-compiler", async move {
            log::info!("CompileActor: initialized");
            while let Some(event) = tokio::select! {
                Some(it) = fs_rx.recv() => Some(CompilerInterrupt::Fs(it)),
                Some(it) = self.memory_recv.recv() => Some(CompilerInterrupt::Memory(it)),
                Some(it) = self.steal_recv.recv() => Some(CompilerInterrupt::Task(it)),
            } {
                let mut need_recompile = false;
                need_recompile = self.process(event) || need_recompile;
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
                    need_recompile = self.process(event) || need_recompile;
                }

                if need_recompile {
                    self.compile(&compiler_ack);
                }
            }
        })
        .unwrap();

        Some(compile_thread)
    }

    pub async fn block_run(mut self) -> bool {
        if !self.enable_watch {
            let compiled = self
                .compiler
                .with_stage_diag::<false, _>("compiling", |driver| driver.compile());
            return compiled.is_some();
        }

        if let Some(h) = self.spawn().await {
            h.join().unwrap();
        }

        true
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
pub struct CompileClient<Ctx> {
    steal_send: mpsc::UnboundedSender<BorrowTask<Ctx>>,
    memory_send: mpsc::UnboundedSender<MemoryEvent>,

    _ctx: std::marker::PhantomData<Ctx>,
}

impl<Ctx> CompileClient<Ctx> {
    fn steal_inner<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx) -> Ret + Send + 'static,
    ) -> oneshot::Receiver<Ret> {
        let (tx, rx) = oneshot::channel();

        self.steal_send
            .send(Box::new(move |this: &mut Ctx| {
                if tx.send(f(this)).is_err() {
                    // Receiver was dropped. The main thread may have exited, or the request may
                    // have been cancelled.
                    log::warn!("could not send back return value from Typst thread");
                }
            }))
            .unwrap();
        rx
    }

    pub fn steal<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx) -> Ret + Send + 'static,
    ) -> ZResult<Ret> {
        Ok(self.steal_inner(f).blocking_recv().unwrap())
    }

    /// Steal the compiler thread and run the given function.
    pub async fn steal_async<Ret: Send + 'static>(
        &mut self,
        f: impl FnOnce(&mut Ctx, tokio::runtime::Handle) -> Ret + Send + 'static,
    ) -> ZResult<Ret> {
        // get current async handle
        let handle = tokio::runtime::Handle::current();
        Ok(self
            .steal_inner(move |this: &mut Ctx| f(this, handle.clone()))
            .await
            .unwrap())
    }

    pub fn add_memory_changes(&self, event: MemoryEvent) {
        self.memory_send.send(event).unwrap();
    }
}

#[derive(Debug, Serialize)]
pub struct DocToSrcJumpInfo {
    filepath: String,
    start: Option<(usize, usize)>, // row, column
    end: Option<(usize, usize)>,
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

    pub async fn resolve_doc_to_src_jump(&mut self, id: u64) -> ZResult<Option<DocToSrcJumpInfo>> {
        let resolve_off =
            |src: &Source, off: usize| src.byte_to_line(off).zip(src.byte_to_column(off));

        self.steal_async(move |this, _| {
            let world = this.compiler.world();
            let span = span_id_from_u64(id)?;
            let src_id = span.id()?;
            let source = world.source(src_id).ok()?;
            let range = source.find(span)?.range();
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
                page: NonZeroUsize::new(i + 1).unwrap(),
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
        page: NonZeroUsize::new(ppage + 1).unwrap(),
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
