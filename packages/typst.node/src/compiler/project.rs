use std::sync::{Arc, Mutex};

use reflexo_typst::vfs::notify::NotifyMessage;
use tokio::sync::mpsc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use reflexo_typst::error::WithContextUntyped;
use reflexo_typst::{watch_deps, SystemCompilerFeat, TypstSystemUniverse};
use tinymist_project::{
    CompileHandler, CompileServerOpts, Interrupt, ProjectCompiler as ProjectCompilerBase,
};

use super::{create_universe, NodeCompileArgs};
use crate::error::{map_node_error, NodeError};

type WatchFunction = ThreadsafeFunction<u32, ErrorStrategy::Fatal>;

/// Project watcher.
#[napi]
pub struct ProjectWatcher {
    tx: mpsc::UnboundedSender<Message>,
}

#[napi]
impl ProjectWatcher {
    /// Creates a new compiler based on the given arguments.
    ///
    /// == Example
    ///
    /// Creates a new compiler with default arguments:
    /// ```ts
    /// const compiler = ProjectCompiler.create();
    /// ```
    ///
    /// Creates a new compiler with custom arguments:
    /// ```ts
    /// const compiler = ProjectCompiler.create({
    ///   workspace: '/path/to/workspace',
    /// });
    /// ```
    #[napi]
    pub fn create(args: Option<NodeCompileArgs>) -> Result<ProjectWatcher, NodeError> {
        let driver = create_universe(args).map_err(map_node_error)?;
        let (tx, rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            let worker = ProjectBackgroundWorker::new(driver, rx);

            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(worker.run());
        });
        Ok(ProjectWatcher { tx })
    }

    /// Watches multiple documents for changes.
    ///
    /// == Example
    ///
    /// Watches and updates multiple documents for changes:
    /// ```ts
    /// const compiler = ProjectCompiler.create();
    /// compiler.update(['a.typ', 'b.typ']);
    /// compiler.watch(console.log);
    ///
    /// compiler.update([{ main: 'a.typ', workspace: '..' }]);
    /// // watch again will flush changes cancel the previous watch
    /// compiler.watch(console.log);
    /// ```
    ///
    /// Glob patterns watches:
    /// ```ts
    /// const watch = require('glob-watcher');
    /// const compiler = ProjectCompiler.create();
    ///
    /// const onChange = (project) => console.log(project);
    ///
    /// const watcher = watch(['./*.typ', '!./something.typ']);
    /// watcher.on('add', (path) => {
    ///   compiler.add(path); compiler.watch(onChange);
    /// });
    /// watcher.on('remove', (path) => {
    ///   compiler.remove(path); compiler.watch(onChange);
    /// });
    /// ```
    #[napi(ts_args_type = "callback: (project: any) => void")]
    pub fn watch(&self, callback: JsFunction) -> Result<(), NodeError> {
        let tsfn: WatchFunction = callback
            .create_threadsafe_function(0, |ctx| {
                ctx.env.create_uint32(ctx.value + 1).map(|v| vec![v])
            })
            .context_ut("failed to create threadsafe function")
            .map_err(map_node_error)?;
        self.tx
            .send(Message::Watch(tsfn))
            .map_err(|_| "send watch message failed")
            .context_ut("failed to watch")
            .map_err(map_node_error)?;

        Ok(())
    }

    /// Adds multiple documents to the compiler.
    #[napi]
    pub fn add(&self, items: serde_json::Value) -> Result<(), NodeError> {
        self.tx
            .send(Message::Add(convert_items(items)?))
            .map_err(|_| "send add message failed")
            .context_ut("failed to add")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Updates multiple documents in the compiler.
    #[napi]
    pub fn update(&self, items: serde_json::Value) -> Result<(), NodeError> {
        self.tx
            .send(Message::Update(convert_items(items)?))
            .map_err(|_| "send update message failed")
            .context_ut("failed to update")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Removes multiple documents from the compiler.
    #[napi]
    pub fn remove(&self, items: serde_json::Value) -> Result<(), NodeError> {
        self.tx
            .send(Message::Remove(convert_items(items)?))
            .map_err(|_| "send remove message failed")
            .context_ut("failed to remove")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Gets the list of documents in the compiler.
    #[napi]
    pub fn list(&self) -> Result<Vec<String>, NodeError> {
        Ok(vec![])
    }
}

fn convert_items(_items: serde_json::Value) -> Result<Vec<String>, NodeError> {
    todo!()
}

enum Message {
    Add(Vec<String>),
    Update(Vec<String>),
    Remove(Vec<String>),
    Watch(WatchFunction),
}

struct ProjectBackgroundWorker {
    compiler: ProjectCompilerBase<SystemCompilerFeat, ()>,
    dep_rx: mpsc::UnboundedReceiver<NotifyMessage>,
    rx: mpsc::UnboundedReceiver<Message>,
    handler: Arc<ProjectHandler>,
}

impl ProjectBackgroundWorker {
    fn new(verse: TypstSystemUniverse, rx: mpsc::UnboundedReceiver<Message>) -> Self {
        let (dep_tx, dep_rx) = mpsc::unbounded_channel();

        let handler = Arc::new(ProjectHandler {
            watch: Arc::new(Mutex::new(None)),
        });

        let compiler = ProjectCompilerBase::new(
            verse,
            dep_tx,
            CompileServerOpts {
                handler: handler.clone() as Arc<_>,
                enable_watch: true,
            },
        );

        Self {
            compiler,
            dep_rx,
            rx,
            handler,
        }
    }

    async fn run(mut self) {
        let (intr_tx, mut intr_rx) = mpsc::unbounded_channel();

        tokio::spawn(watch_deps(self.dep_rx, move |evt| {
            intr_tx.send(Interrupt::<SystemCompilerFeat>::Fs(evt)).ok();
        }));

        let _ = intr_rx;
        loop {
            tokio::select! {
                Some(intr) = intr_rx.recv() => {
                    self.compiler.process(intr);
                }
                Some(msg) = self.rx.recv() => {
                    match msg {
                        Message::Watch(watch) => {
                            *self.handler.watch.lock().unwrap() = Some(watch);
                        }
                        Message::Add(items) => {
                            let _ = items;
                        }
                        Message::Update(items) => {
                            let _ = items;
                        }
                        Message::Remove(items) => {
                            let _ = items;
                        }
                    }
                }
            }
        }
    }
}

struct ProjectHandler {
    watch: Arc<Mutex<Option<WatchFunction>>>,
}

impl CompileHandler<SystemCompilerFeat, ()> for ProjectHandler {
    fn on_any_compile_reason(&self, _state: &mut ProjectCompilerBase<SystemCompilerFeat, ()>) {
        self.watch
            .lock()
            .unwrap()
            .as_ref()
            .map(|watch| watch.call(0, ThreadsafeFunctionCallMode::NonBlocking));
    }

    fn notify_compile(
        &self,
        _res: &tinymist_project::CompiledArtifact<SystemCompilerFeat>,
        _rep: tinymist_project::CompileReport,
    ) {
        todo!()
    }

    fn status(
        &self,
        _revision: usize,
        _id: &reflexo_typst::ProjectInsId,
        _rep: tinymist_project::CompileReport,
    ) {
        todo!()
    }
}
