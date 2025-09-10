use std::sync::{Arc, Mutex};

use reflexo_typst::hash::FxHashMap;
use reflexo_typst::system::SystemWorldComputeGraph;
use reflexo_typst::vfs::notify::NotifyMessage;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use reflexo_typst::error::WithContextUntyped;
use reflexo_typst::{
    error_once, watch_deps, ArcInto, Bytes, CompilationTask, CompileSnapshot, DocumentQuery,
    EntryReader, EntryState, ExportComputation, ExportWebSvgModuleTask, FlagTask, ProjectInsId,
    SystemCompilerFeat, TaskInputs, TypstDocument, TypstDocumentTrait, TypstPagedDocument,
    TypstSystemUniverse, TypstSystemWorld, MEMORY_MAIN_ENTRY,
};
use tinymist_project::{
    CompileHandler, CompileServerOpts, CompileSignal, CompiledArtifact, Interrupt,
    ProjectCompiler as ProjectCompilerBase,
};

use super::{abs_user_path, create_inputs, create_universe, CompileArgs};
use crate::{error::*, NodeTypstDocument};
use crate::{CompileDocArgs, QueryDocArgs};

type WatchFunction = Arc<ThreadsafeFunction<NodeTypstProject, ErrorStrategy::Fatal>>;

/// Project watcher.
#[napi]
pub struct ProjectWatcher {
    entry: EntryState,
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
    pub fn create(args: Option<CompileArgs>) -> Result<ProjectWatcher, NodeError> {
        let verse = create_universe(args).map_err(map_node_error)?;
        let entry = verse.entry_state();
        let (tx, rx) = mpsc::unbounded_channel();
        std::thread::spawn(move || {
            let worker = ProjectBackgroundWorker::new(verse, rx);

            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(worker.run());
        });
        Ok(ProjectWatcher { entry, tx })
    }

    /// Evict the **global** cache.
    ///
    /// This removes all memoized results from the cache whose age is larger
    /// than or equal to `max_age`. The age of a result grows by one during
    /// each eviction and is reset to zero when the result produces a cache
    /// hit. Set `max_age` to zero to completely clear the cache.
    ///
    /// A suggested `max_age` value for regular non-watch tools is `10`.
    /// A suggested `max_age` value for regular watch tools is `30`.
    #[napi]
    pub fn evict_cache(&mut self, max_age: u32) -> Result<(), NodeError> {
        self.tx
            .send(Message::EvictCache(max_age))
            .map_err(|_| "send watch message failed")
            .context_ut("failed to watch")
            .map_err(map_node_error)
    }

    /// Watches multiple documents for changes.
    ///
    /// == Example
    ///
    /// Watches and updates multiple documents for changes:
    /// ```ts
    /// const compiler = ProjectCompiler.create();
    /// compiler.update(['a.typ', 'b.typ'], console.log);
    /// compiler.watch();
    ///
    /// compiler.update([{ main: 'a.typ', workspace: '..' }], console.log);
    /// // watch again will flush changes cancel the previous watch
    /// compiler.watch();
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
    ///   compiler.add(path, onChange); compiler.watch();
    /// });
    /// watcher.on('remove', (path) => {
    ///   compiler.remove(path, onChange); compiler.watch();
    /// });
    /// ```
    #[napi]
    pub fn watch(&self) -> Result<(), NodeError> {
        self.tx
            .send(Message::Watch)
            .map_err(|_| "send watch message failed")
            .context_ut("failed to watch")
            .map_err(map_node_error)?;

        Ok(())
    }

    /// Adds multiple documents to the compiler.
    #[napi(
        ts_args_type = "items: types.ProjectWatchItems, callback: (project: NodeTypstProject) => void"
    )]
    pub fn add(&self, items: serde_json::Value, callback: JsFunction) -> Result<(), NodeError> {
        let tsfn: WatchFunction = Arc::new(
            callback
                .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
                .context_ut("failed to create threadsafe function")
                .map_err(map_node_error)?,
        );
        self.tx
            .send(Message::Add(convert_items(items, &self.entry)?, tsfn))
            .map_err(|_| "send add message failed")
            .context_ut("failed to add")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Updates multiple documents in the compiler.
    #[napi(
        ts_args_type = "items: types.ProjectWatchItems, callback: (project: NodeTypstProject) => void"
    )]
    pub fn update(&self, items: serde_json::Value, callback: JsFunction) -> Result<(), NodeError> {
        let tsfn: WatchFunction = Arc::new(
            callback
                .create_threadsafe_function(0, |ctx| Ok(vec![ctx.value]))
                .context_ut("failed to create threadsafe function")
                .map_err(map_node_error)?,
        );
        self.tx
            .send(Message::Update(convert_items(items, &self.entry)?, tsfn))
            .map_err(|_| "send update message failed")
            .context_ut("failed to update")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Removes multiple documents from the compiler.
    #[napi(ts_args_type = "items: types.ProjectWatchItems")]
    pub fn remove(&self, items: serde_json::Value) -> Result<(), NodeError> {
        self.tx
            .send(Message::Remove(convert_items(items, &self.entry)?))
            .map_err(|_| "send remove message failed")
            .context_ut("failed to remove")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Clears all documents in the compiler.
    #[napi]
    pub fn clear(&self) -> Result<(), NodeError> {
        self.tx
            .send(Message::Clear)
            .map_err(|_| "send clear message failed")
            .context_ut("failed to clear")
            .map_err(map_node_error)?;
        Ok(())
    }

    /// Gets the list of documents in the compiler.
    #[napi]
    pub fn list(&self) -> Result<Vec<String>, NodeError> {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
struct Item {
    main: String,
    workspace: Option<String>,
}

impl Item {
    fn select_in(&self, base: &EntryState) -> reflexo_typst::Result<EntryState> {
        let main = abs_user_path(self.main.as_str())?;

        let entry = match &self.workspace {
            Some(workspace) => {
                let workspace_dir = abs_user_path(workspace.as_str())?;
                EntryState::new_workspace(workspace_dir.into())
                    .try_select_path_in_workspace(&main)?
            }
            None => base.try_select_path_in_workspace(&main)?,
        };
        entry.ok_or_else(|| error_once!("the item is not a valid entry"))
    }
}

fn convert_items(
    items: serde_json::Value,
    base: &EntryState,
) -> Result<Vec<EntryState>, NodeError> {
    match items {
        item @ serde_json::Value::String(..) => Ok(vec![resolve_item(item, base)?]),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|item| resolve_item(item, base))
            .collect::<Result<Vec<_>, _>>(),
        _ => Err(error_once!("invalid items type, expect string or array"))
            .map_err(map_node_error)?,
    }
}

fn resolve_item(item: serde_json::Value, base: &EntryState) -> Result<EntryState, NodeError> {
    convert_item(item)?.select_in(base).map_err(map_node_error)
}

fn convert_item(item: serde_json::Value) -> Result<Item, NodeError> {
    Ok(match item {
        serde_json::Value::String(item) => Item {
            main: item.clone(),
            workspace: None,
        },
        value => serde_json::from_value(value)
            .context_ut("failed to convert watch item")
            .map_err(map_node_error)?,
    })
}

enum Message {
    EvictCache(u32),

    Add(Vec<EntryState>, WatchFunction),
    Update(Vec<EntryState>, WatchFunction),
    Remove(Vec<EntryState>),
    Clear,
    Watch,
}

struct ProjectBackgroundWorker {
    compiler: ProjectCompilerBase<SystemCompilerFeat, ProjectInsStateExt>,
    dep_rx: mpsc::UnboundedReceiver<NotifyMessage>,

    intr_tx: mpsc::UnboundedSender<Interrupt<SystemCompilerFeat>>,
    intr_rx: mpsc::UnboundedReceiver<Interrupt<SystemCompilerFeat>>,

    rx: mpsc::UnboundedReceiver<Message>,
    // todo: rpds
    view: FxHashMap<EntryState, WatchFunction>,
    handler: Arc<ProjectHandler>,
}

impl ProjectBackgroundWorker {
    fn new(verse: TypstSystemUniverse, rx: mpsc::UnboundedReceiver<Message>) -> Self {
        let (intr_tx, intr_rx) = mpsc::unbounded_channel();
        let (dep_tx, dep_rx) = mpsc::unbounded_channel();

        let handler = Arc::new(ProjectHandler {
            intr_tx: intr_tx.clone(),
            watch: Arc::default(),
        });

        let compiler = ProjectCompilerBase::new(
            verse,
            dep_tx,
            CompileServerOpts {
                handler: handler.clone() as Arc<_>,
                export_target: tinymist_project::ExportTarget::Html,
                ignore_first_sync: false,
            },
        );

        Self {
            compiler,
            intr_tx,
            intr_rx,
            dep_rx,
            rx,
            handler,
            view: FxHashMap::default(),
        }
    }

    async fn run(mut self) {
        let intr_tx = self.intr_tx.clone();
        let mut intr_rx = self.intr_rx;

        tokio::spawn(watch_deps(self.dep_rx, move |evt| {
            intr_tx.send(Interrupt::<SystemCompilerFeat>::Fs(evt)).ok();
        }));

        loop {
            let msg = tokio::select! {
                Some(intr) = intr_rx.recv() => {

                    if let Interrupt::Compiled(compiled) = &intr {
                        let proj = self.compiler.projects().find(|p| &p.id == compiled.id());
                        if let Some(proj) = proj {
                            proj.ext.is_compiling = false;
                            proj.ext.last_compilation = Some(compiled.clone());
                        }
                    }

                    self.compiler.process(intr);
                    continue;
                }
                Some(msg) = self.rx.recv() => msg,
                else => break,
            };

            match msg {
                Message::EvictCache(max_age) => {
                    let max_age = usize::try_from(max_age).unwrap();
                    comemo::evict(max_age);

                    for proj in self.compiler.projects() {
                        proj.verse.evict(max_age);
                    }
                }

                Message::Watch => {
                    let view = &self.view;
                    let mut watch_fns = self.handler.watch.lock().unwrap();

                    self.compiler.clear_dedicates();
                    for (idx, (entry, watch_fn)) in view.iter().enumerate() {
                        let id = format!("project-{idx}");

                        // todo: html
                        let id = self.compiler.restart_dedicate(&id, entry.clone());

                        match id {
                            Ok(id) => {
                                watch_fns.insert(id, watch_fn.clone());
                            }
                            Err(e) => {
                                // todo: error handler
                                eprintln!("failed to restart project: {e}");
                            }
                        }
                    }

                    self.compiler
                        .handler
                        .clone()
                        .on_any_compile_reason(&mut self.compiler);
                }
                Message::Add(items, tsfn) => {
                    for item in items {
                        self.view.insert(item, tsfn.clone());
                    }
                }
                Message::Update(items, tsfn) => {
                    self.view.clear();
                    for item in items {
                        self.view.insert(item, tsfn.clone());
                    }
                }
                Message::Remove(items) => {
                    for item in items {
                        self.view.remove(&item);
                    }
                }
                Message::Clear => {
                    self.view.clear();
                }
            };
        }
        eprintln!("exit");
    }
}

/// Either a compiled document or compile arguments.
type MayCompileOpts<'a> = Either<&'a NodeTypstDocument, CompileDocArgs>;

#[napi]
pub struct NodeTypstProject {
    graph: Arc<SystemWorldComputeGraph>,
}

// todo: merge me with NodeCompiler.
#[napi]
impl NodeTypstProject {
    /// Gets the inner world.
    fn spawn_world(&self) -> TypstSystemWorld {
        self.graph.snap.world.clone()
    }

    /// Compiles the document as paged target.
    #[napi]
    pub fn compile(&mut self, opts: CompileDocArgs) -> Result<NodeTypstCompileResult, NodeError> {
        self.compile_raw::<reflexo_typst::TypstPagedDocument>(opts)
    }

    /// Compiles the document as html target.
    #[napi]
    pub fn compile_html(
        &mut self,
        opts: CompileDocArgs,
    ) -> Result<NodeTypstCompileResult, NodeError> {
        self.compile_raw::<reflexo_typst::TypstHtmlDocument>(opts)
    }

    // todo: tinymist_world implement it.
    /// Create a snapshoted world by typst.node's [`CompileDocArgs`].
    /// Should not affect the current universe (global state).
    pub fn computation(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> reflexo_typst::Result<Arc<SystemWorldComputeGraph>, NodeError> {
        use reflexo_typst::ShadowApi;
        use reflexo_typst::TypstWorld;

        let graph = &self.graph;

        // Convert the input pairs to a dictionary.
        let inputs = compile_by.inputs.map(create_inputs);
        if let Some(main_file_content) = compile_by.main_file_content {
            if compile_by.main_file_path.is_some() {
                return Err(error_once!(
                    "main file content and path cannot be specified at the same time"
                ))?;
            }

            let world = graph.snap.world.clone();

            let mut world = if world.main_id().is_some() {
                world.task(TaskInputs {
                    entry: None,
                    inputs,
                })
            } else {
                let world = world.task(TaskInputs {
                    entry: Some(
                        world
                            .entry_state()
                            .select_in_workspace(MEMORY_MAIN_ENTRY.vpath().as_rooted_path()),
                    ),
                    inputs,
                });

                world
            };

            world
                .map_shadow_by_id(world.main(), Bytes::new(main_file_content))
                .unwrap();
            if compile_by.reset_read.unwrap_or(true) {
                world.reset_read();
            }
            let snap = CompileSnapshot::from_world(world);

            return Ok(SystemWorldComputeGraph::new(snap));
        };

        let entry = if let Some(main_file_path) = compile_by.main_file_path {
            if compile_by.main_file_content.is_some() {
                return Err(error_once!(
                    "main file content and path cannot be specified at the same time"
                ))?;
            }

            let abs_fp = std::path::absolute(main_file_path.as_str());
            let fp = abs_fp
                .as_ref()
                .map(std::path::Path::new)
                .map_err(|e| error_once!("cannot absolutize the main file path", err: e))?;
            graph
                .snap
                .world
                .entry_state()
                .try_select_path_in_workspace(fp)?
        } else {
            None
        };

        let mut snap = graph.snap.clone().task(TaskInputs { entry, inputs });
        if compile_by.reset_read.unwrap_or(true) {
            snap.world.reset_read();
        }
        Ok(SystemWorldComputeGraph::new(snap))
    }

    /// Compiles the document as paged target.
    pub fn compile_raw<
        D: reflexo_typst::TypstDocumentTrait + ArcInto<TypstDocument> + Send + Sync + 'static,
    >(
        &mut self,
        opts: CompileDocArgs,
    ) -> Result<NodeTypstCompileResult, NodeError> {
        let result = self.compile_raw2::<D>(opts);
        Ok(result.map_err(map_node_error)?.into())
    }

    pub fn compile_raw2<
        D: reflexo_typst::TypstDocumentTrait + ArcInto<TypstDocument> + Send + Sync + 'static,
    >(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> reflexo_typst::Result<ExecResultRepr<NodeTypstDocument>, NodeError> {
        let graph = self.computation(compile_by)?;

        let _ = graph.provide::<FlagTask<CompilationTask<D>>>(Ok(FlagTask::flag(true)));
        let result = graph.compute::<CompilationTask<D>>()?;
        let result: ExecResultRepr<Arc<D>> = result.as_ref().clone().expect("enabled").into();

        Ok(result
            .map(|d| NodeTypstDocument {
                graph: graph.clone(),
                doc: d.arc_into(),
            })
            .with_graph(graph))
    }

    /// Fetches the diagnostics of the document.
    #[napi]
    pub fn fetch_diagnostics(
        &mut self,
        opts: &NodeError,
    ) -> Result<Vec<serde_json::Value>, NodeError> {
        opts.get_json_diagnostics(Some(&self.spawn_world()))
    }

    /// Queries the data of the document.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs, args: QueryDocArgs")]
    pub fn query(
        &mut self,
        opts: MayCompileOpts,
        args: QueryDocArgs,
    ) -> Result<serde_json::Value, NodeError> {
        let doc = self.may_compile::<TypstPagedDocument>(opts)?;

        let config = reflexo_typst::task::QueryTask {
            export: reflexo_typst::task::ExportTask::default(),
            format: "json".to_owned(),
            output_extension: None,
            selector: args.selector,
            field: args.field,
            one: false,
        };

        DocumentQuery::doc_get_as_value(&doc.graph, &doc.doc, &config).map_err(map_node_error)
    }

    /// Compiles the document as a specific type.
    pub fn may_compile<D: TypstDocumentTrait + Send + Sync + 'static>(
        &mut self,
        opts: MayCompileOpts,
    ) -> Result<NodeTypstDocument, NodeError>
    where
        Arc<D>: Into<TypstDocument>,
    {
        Ok(match opts {
            MayCompileOpts::A(doc) => doc.clone(),
            MayCompileOpts::B(compile_by) => {
                let mut res = self.compile_raw::<D>(compile_by)?;
                if let Some(diagnostics) = res.take_diagnostics() {
                    // todo: format diagnostics
                    return Err(Error::from_status(diagnostics));
                }

                res.result().unwrap()
            }
        })
    }

    /// Compiles the document as a specific type.
    pub fn may_compile2<D: TypstDocumentTrait + Send + Sync + 'static>(
        &mut self,
        opts: MayCompileOpts,
    ) -> std::result::Result<ExecResultRepr<NodeTypstDocument>, NodeError>
    where
        Arc<D>: Into<TypstDocument>,
    {
        Ok(match opts {
            MayCompileOpts::A(doc) => doc.clone().into(),
            MayCompileOpts::B(compile_by) => self.compile_raw2::<D>(compile_by)?,
        })
    }

    /// Compiles the document as a specific type.
    pub fn compile_as<
        T: ExportComputation<SystemCompilerFeat, reflexo_typst::TypstPagedDocument>,
        RO: From<T::Output>,
    >(
        &mut self,
        opts: MayCompileOpts,
        config: &T::Config,
    ) -> Result<RO, NodeError> {
        let doc = self.may_compile::<reflexo_typst::TypstPagedDocument>(opts)?;
        T::cast_run(&doc.graph, &doc.doc, config)
            .map_err(map_node_error)
            .map(From::from)
    }

    /// Compiles the document as a specific type.
    pub fn compile_as_html<
        T: ExportComputation<SystemCompilerFeat, reflexo_typst::TypstHtmlDocument>,
        RO: From<T::Output>,
    >(
        &mut self,
        opts: MayCompileOpts,
        config: &T::Config,
    ) -> std::result::Result<ExecResultRepr<RO>, NodeError> {
        let doc = self.may_compile2::<reflexo_typst::TypstHtmlDocument>(opts)?;
        Ok(doc.and_then(|doc| Ok(T::cast_run(&doc.graph, &doc.doc, config)?.into())))
    }

    /// Compiles the document as buffer.
    pub fn compile_as_buffer<
        T: ExportComputation<SystemCompilerFeat, reflexo_typst::TypstPagedDocument, Output = Bytes>,
    >(
        &mut self,
        opts: MayCompileOpts,
        config: &T::Config,
    ) -> Result<Buffer, NodeError> {
        let res = self.compile_as::<T, Bytes>(opts, config)?;
        Ok(Buffer::from(res.as_slice()))
    }

    /// Simply compiles the document as a vector IR.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs")]
    pub fn vector(&mut self, compiled_or_by: MayCompileOpts) -> Result<Buffer, NodeError> {
        use reflexo_vec2svg::DefaultExportFeature;
        type Export = reflexo_typst::WebSvgModuleExport<DefaultExportFeature>;
        self.compile_as_buffer::<Export>(compiled_or_by, &ExportWebSvgModuleTask::default())
    }

    /// Simply compiles the document as a PDF.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs, opts?: RenderPdfOpts")]
    #[cfg(feature = "pdf")]
    pub fn pdf(
        &mut self,
        compiled_or_by: MayCompileOpts,
        opts: Option<crate::RenderPdfOpts>,
    ) -> Result<Buffer, NodeError> {
        type Export = reflexo_typst::PdfExport;
        use reflexo_typst::{error::WithContext, task::ExportPdfTask};

        let e = if let Some(opts) = opts {
            let creation_timestamp = opts.creation_timestamp;

            let standard = opts
                .pdf_standard
                .map(|single| serde_json::from_value(serde_json::Value::String(single)))
                .transpose()
                .context("failed to deserialize PdfStandard for typst")
                .map_err(map_node_error)?;

            ExportPdfTask {
                export: Default::default(),
                pdf_standards: standard.into_iter().collect(),
                creation_timestamp,
            }
        } else {
            ExportPdfTask::default()
        };

        self.compile_as_buffer::<Export>(compiled_or_by, &e)
    }

    /// Simply compiles the document as a plain SVG.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs")]
    #[cfg(feature = "svg")]
    pub fn plain_svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        use reflexo_typst::task::ExportSvgTask;

        type Export = reflexo_typst::SvgExport;
        self.compile_as::<Export, _>(compiled_or_by, &ExportSvgTask::default())
    }

    /// Simply compiles the document as a rich-contented SVG (for browsers).
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs")]
    #[cfg(feature = "svg")]
    pub fn svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        use reflexo_typst::ExportWebSvgTask;
        use reflexo_vec2svg::DefaultExportFeature;

        type Export = reflexo_typst::WebSvgExport<DefaultExportFeature>;
        self.compile_as::<Export, _>(compiled_or_by, &ExportWebSvgTask::default())
    }

    // todo: when feature is disabled, it results a compile error
    /// Simply compiles the document as a HTML.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs")]
    #[cfg(feature = "html")]
    pub fn html(&mut self, compiled_or_by: MayCompileOpts) -> Result<Option<String>, NodeError> {
        use reflexo_typst::ExportStaticHtmlTask;

        type Export = reflexo_typst::StaticHtmlExport;
        self.compile_as_html::<Export, _>(compiled_or_by, &ExportStaticHtmlTask::default())
            .map_err(map_node_error)?
            .to_napi_result()
    }

    /// Compiles the document as a HTML.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocArgs")]
    #[cfg(feature = "html")]
    pub fn try_html(&mut self, compiled_or_by: MayCompileOpts) -> NodeHtmlOutputExecResult {
        use reflexo_typst::ExportHtmlTask;

        use crate::NodeHtmlOutput;

        type Export = reflexo_typst::HtmlOutputExport;
        let res = self
            .compile_as_html::<Export, _>(compiled_or_by, &ExportHtmlTask::default())
            .map(|res| {
                res.flatten().map(|inner| NodeHtmlOutput {
                    inner: Arc::new(inner),
                })
            });
        ExecResultRepr::from_result(res).into()
    }
}

#[derive(Default)]
pub struct ProjectInsStateExt {
    pub is_compiling: bool,
    pub last_compilation: Option<CompiledArtifact<SystemCompilerFeat>>,
}

struct ProjectHandler {
    intr_tx: mpsc::UnboundedSender<Interrupt<SystemCompilerFeat>>,
    watch: Arc<Mutex<FxHashMap<ProjectInsId, WatchFunction>>>,
}

impl CompileHandler<SystemCompilerFeat, ProjectInsStateExt> for ProjectHandler {
    fn on_any_compile_reason(
        &self,
        state: &mut ProjectCompilerBase<SystemCompilerFeat, ProjectInsStateExt>,
    ) {
        // let Some(watch) = self.watch.lock().unwrap().get(&state);

        for proj in state.projects() {
            const VFS_SUB: CompileSignal = CompileSignal {
                by_mem_events: true,
                by_fs_events: true,
                by_entry_update: false,
            };

            let reason = proj.reason;

            let is_vfs_sub = reason.any() && !reason.exclude(VFS_SUB).any();
            let id = &proj.id;

            if is_vfs_sub
                && 'vfs_is_clean: {
                    let Some(compilation) = &proj.ext.last_compilation else {
                        break 'vfs_is_clean false;
                    };

                    let last_rev = compilation.world().vfs().revision();
                    let deps = compilation.depended_files().clone();
                    proj.verse.vfs().is_clean_compile(last_rev.get(), &deps)
                }
            {
                eprintln!("Project: skip compilation for {id:?} due to harmless vfs changes");
                proj.reason = CompileSignal::default();
                continue;
            }

            let id = proj.id.clone();
            let intr_tx = self.intr_tx.clone();
            let watches = self.watch.clone();

            let Some(may_compile) =
                proj.may_compile2(move |graph: &Arc<SystemWorldComputeGraph>| {
                    // todo: don't do this aggressively but we do want to update deps by that
                    let res = CompiledArtifact::from_graph(graph.clone(), true);

                    let watches = watches.lock().unwrap();
                    let f = watches.get(&id);
                    if let Some(f) = f {
                        let status: Status = f.call(
                            NodeTypstProject {
                                graph: graph.clone(),
                            },
                            ThreadsafeFunctionCallMode::Blocking,
                        );
                        if !matches!(status, Status::Ok) {
                            eprintln!("failed to call watch function: {status:?}");
                        }

                        intr_tx
                            .send(Interrupt::<SystemCompilerFeat>::Compiled(res))
                            .ok();
                    }
                })
            else {
                continue;
            };

            proj.ext.is_compiling = true;
            rayon::spawn(move || {
                may_compile();
            })
        }
    }

    fn notify_compile(&self, _res: &tinymist_project::CompiledArtifact<SystemCompilerFeat>) {}

    fn status(&self, _revision: usize, _rep: tinymist_project::CompileReport) {}
}
