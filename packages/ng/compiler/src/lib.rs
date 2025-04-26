use std::{ops::Deref, path::Path, sync::Arc};

use js_sys::{Object, Uint8Array};
use reflexo_typst::{
    font::web::BrowserFontSearcher,
    foundations::IntoValue,
    package::registry::{JsRegistry, ProxyContext},
    vfs::browser::ProxyAccessModel,
    BrowserCompilerFeat, Bytes, CompileSnapshot, EntryReader, ExportComputation, LazyHash,
    TaskInputs, TypstBrowserUniverse, TypstBrowserWorld, TypstHtmlDocument, TypstPagedDocument,
    WorldComputeGraph,
};
use typst::{diag::SourceDiagnostic, ecow::EcoVec};
use wasm_bindgen::{
    convert::{FromWasmAbi, IntoWasmAbi, RefFromWasmAbi},
    prelude::*,
};

enum Compiler {
    Root(Root),
    Snapshot(Snapshot),
}

struct Root {
    verse: TypstBrowserUniverse,
}

#[derive(Clone)]
struct Snapshot {
    world: TypstBrowserWorld,
}

impl Snapshot {
    fn make_doc(self) -> Result<TypstDocument, JsValue> {
        Ok(TypstDocument {
            world: { WorldComputeGraph::new(CompileSnapshot::from_world(self.world)) },
            paged: None,
            html: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        })
    }
}

#[wasm_bindgen]
pub struct TypstCompiler(Compiler);

impl Default for TypstCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TypstCompiler {
    /// Creates a new instance of the TypstCompiler.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let am = ProxyAccessModel {
            context: wasm_bindgen::JsValue::UNDEFINED,
            mtime_fn: js_sys::Function::new_no_args("return 0"),
            is_file_fn: js_sys::Function::new_no_args("return true"),
            real_path_fn: js_sys::Function::new_with_args("path", "return path"),
            read_all_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy AccessModel, please initialize compiler with withAccessModel()')",
            ),
        };
        let registry = JsRegistry {
            context: ProxyContext::new(wasm_bindgen::JsValue::UNDEFINED),
            real_resolve_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy Registry, please initialize compiler with withPackageRegistry()')",
            ),
        };
        let fonts = BrowserFontSearcher::default().build();

        let verse = TypstBrowserUniverse::new("/".into(), None, am, registry, fonts);

        Self(Compiler::Root(Root { verse }))
    }

    /// Creates a snapshot for new task.
    pub fn task(&self, args: Option<SnapshotArgs>) -> Result<Self, JsValue> {
        Ok(Self(Compiler::Snapshot(self.to_snapshot(args)?)))
    }

    /// Runs a typst task.
    pub fn run(
        &mut self,
        input_kind: u8,
        input: &Object,
        task_kind: u8,
        task: &JsValue,
    ) -> Result<JsValue, JsValue> {
        let input_opts = InputOpts::try_from((input_kind, input))?;
        let output_opts = OutputOpts::try_from((task_kind, task))?;
        let mut doc = self.compile_doc_or_ref(input_opts)?;

        self.trigger_compile(&mut doc, output_opts.compile_type())?;

        if doc.as_ref().errors.iter().any(|e| !e.is_empty()) {
            // todo: handle errors
            return Ok({
                let result = js_sys::Object::new();
                js_sys::Reflect::set(&result, &"doc".into(), &doc.to_owned().into_js_value())?;
                result.into()
            });
        }

        // todo: warning is ignored here.
        let artifact_bytes: Bytes = match output_opts {
            OutputOpts::PagedDoc | OutputOpts::HtmlDoc => return Ok(doc.into()),
            OutputOpts::Dummy => Bytes::new(vec![]),
            #[cfg(feature = "svg")]
            OutputOpts::Svg => {
                use reflexo_typst::ExportWebSvgModuleTask;
                use reflexo_typst::WebSvgModuleExport;
                use reflexo_vec2svg::DefaultExportFeature;

                type SvgModuleExport = WebSvgModuleExport<DefaultExportFeature>;
                SvgModuleExport::run(
                    &doc.as_ref().world,
                    doc.expect_paged()?,
                    &ExportWebSvgModuleTask::default(),
                )?
            }
            #[cfg(feature = "pdf")]
            OutputOpts::Pdf => {
                use reflexo_typst::task::ExportPdfTask;
                use reflexo_typst::PdfExport;

                PdfExport::run(
                    &doc.as_ref().world,
                    doc.expect_paged()?,
                    &ExportPdfTask::default(),
                )?
            }
            #[cfg(feature = "html")]
            OutputOpts::Html => {
                use reflexo_typst::task::ExportHtmlTask;
                use reflexo_typst::StaticHtmlExport;

                Bytes::from_string(StaticHtmlExport::run(
                    &doc.as_ref().world,
                    doc.expect_html()?,
                    &ExportHtmlTask::default(),
                )?)
            }
        };

        let v: JsValue = Uint8Array::from(artifact_bytes.as_slice()).into();

        Ok({
            let result = js_sys::Object::new();
            js_sys::Reflect::set(&result, &"result".into(), &v)?;
            result.into()
        })
    }

    fn to_snapshot(&self, args: Option<SnapshotArgs>) -> Result<Snapshot, JsValue> {
        Ok(match &self.0 {
            Compiler::Root(root) => Snapshot {
                world: root
                    .verse
                    .snapshot_with(Self::create_input(args, &root.verse)?),
            },
            Compiler::Snapshot(snap) => Snapshot {
                world: snap
                    .world
                    .task(Self::create_input(args, &snap.world)?.unwrap_or_default()),
            },
        })
    }

    fn create_input(
        args: Option<SnapshotArgs>,
        ctx: &impl EntryReader,
    ) -> Result<Option<TaskInputs>, JsValue> {
        match args {
            None => Ok(None),
            Some(args) => Ok(Some(TaskInputs {
                entry: match args.main_file_path {
                    Some(path) => ctx
                        .entry_state()
                        .try_select_path_in_workspace(Path::new(&path))?,
                    None => None,
                },
                inputs: args
                    .inputs
                    .map(|inputs| Arc::new(LazyHash::new(convert_inputs(&inputs)))),
            })),
        }
    }

    fn compile_doc(&mut self, input: &SnapshotArgs) -> Result<TypstDocument, JsValue> {
        self.to_snapshot(Some(input.clone()))?.make_doc()
    }

    fn compile_doc_or_ref(&mut self, input: InputOpts) -> Result<JsBorrow<TypstDocument>, JsValue> {
        match input {
            InputOpts::CompileDefault => {
                Ok(JsBorrow::Owned(self.compile_doc(&SnapshotArgs::default())?))
            }
            InputOpts::Compile(opts) => Ok(JsBorrow::Owned(self.compile_doc(opts.deref())?)),
            InputOpts::Compiled(doc) => Ok(JsBorrow::Borrowed(doc)),
        }
    }

    fn trigger_compile(
        &mut self,
        doc: &mut JsBorrow<TypstDocument>,
        compile_type: CompileType,
    ) -> Result<(), JsValue> {
        match compile_type {
            CompileType::PagedDoc => {
                if doc.as_ref().paged.is_none() {
                    let mut owned = doc.to_owned();
                    Self::run_compile(
                        &mut owned.paged,
                        &mut owned.warnings,
                        &mut owned.errors,
                        owned.world.world(),
                    );
                    *doc = JsBorrow::Owned(doc.to_owned());
                }
            }
            CompileType::HtmlDoc => {
                if doc.as_ref().html.is_none() {
                    let mut owned = doc.to_owned();
                    Self::run_compile(
                        &mut owned.html,
                        &mut owned.warnings,
                        &mut owned.errors,
                        owned.world.world(),
                    );
                    *doc = JsBorrow::Owned(doc.to_owned());
                }
            }
        }
        Ok(())
    }

    fn run_compile<T: reflexo_typst::TypstDocumentTrait>(
        cell: &mut Option<Option<Arc<T>>>,
        warnings_cell: &mut Vec<EcoVec<SourceDiagnostic>>,
        errors_cell: &mut Vec<EcoVec<SourceDiagnostic>>,
        // Warned<SourceResult<>>
        world: &TypstBrowserWorld,
    ) {
        match cell {
            Some(_) => {}
            None => {
                let result = typst::compile(world);
                let (doc, errors) = match result.output {
                    Ok(doc) => (Some(doc), EcoVec::default()),
                    Err(err) => (None, err),
                };
                if !result.warnings.is_empty() {
                    warnings_cell.push(result.warnings);
                }
                if !errors.is_empty() {
                    errors_cell.push(errors);
                }

                *cell = Some(doc.map(Arc::new));
            }
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Default)]
pub struct SnapshotArgs {
    main_file_path: Option<String>,
    inputs: Option<Vec<js_sys::Array>>,
}

#[wasm_bindgen]
impl SnapshotArgs {
    #[wasm_bindgen(constructor)]
    pub fn new(main_file_path: Option<String>, inputs: Option<Vec<js_sys::Array>>) -> Self {
        Self {
            main_file_path,
            inputs,
        }
    }
}

enum InputOpts {
    CompileDefault,
    Compile(wasm_bindgen::__rt::RcRef<SnapshotArgs>),
    Compiled(wasm_bindgen::__rt::RcRef<TypstDocument>),
}

impl TryFrom<(u8, &Object)> for InputOpts {
    type Error = JsError;

    fn try_from((kind, value): (u8, &Object)) -> Result<Self, Self::Error> {
        Ok(match kind {
            0 => InputOpts::CompileDefault,
            1 => InputOpts::Compile(SnapshotArgs::js_value_as_ref(value)),
            2 => InputOpts::Compiled(TypstDocument::js_value_as_ref(value)),
            _ => return Err(JsError::new("Invalid input kind.")),
        })
    }
}
enum OutputOpts {
    PagedDoc,
    HtmlDoc,
    Dummy,
    #[cfg(feature = "pdf")]
    Pdf,
    #[cfg(feature = "svg")]
    Svg,
    #[cfg(feature = "html")]
    Html,
}

impl OutputOpts {
    fn compile_type(&self) -> CompileType {
        match self {
            OutputOpts::PagedDoc => CompileType::PagedDoc,
            OutputOpts::HtmlDoc => CompileType::HtmlDoc,
            OutputOpts::Dummy => CompileType::PagedDoc,
            #[cfg(feature = "pdf")]
            OutputOpts::Pdf => CompileType::PagedDoc,
            #[cfg(feature = "svg")]
            OutputOpts::Svg => CompileType::PagedDoc,
            #[cfg(feature = "html")]
            OutputOpts::Html => CompileType::HtmlDoc,
        }
    }
}

impl TryFrom<(u8, &JsValue)> for OutputOpts {
    type Error = JsError;

    fn try_from((format, _opts): (u8, &JsValue)) -> Result<Self, Self::Error> {
        match format {
            0 => Ok(OutputOpts::PagedDoc),
            1 => Ok(OutputOpts::HtmlDoc),
            2 => Ok(OutputOpts::Pdf),
            3 => Ok(OutputOpts::Svg),
            // 4 => Ok(ExportOpts::Png),
            5 => Ok(OutputOpts::Html),
            129 => Ok(OutputOpts::Dummy),
            // 3 => Ok(ExportOpts::Png),
            // 4 => Ok(ExportOpts::Html),
            _ => Err(JsError::new("Invalid export format.")),
        }
    }
}

enum CompileType {
    PagedDoc,
    HtmlDoc,
}

#[wasm_bindgen]
#[derive(Clone)]
struct TypstDocument {
    world: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
    paged: Option<Option<Arc<TypstPagedDocument>>>,
    html: Option<Option<Arc<TypstHtmlDocument>>>,
    warnings: Vec<EcoVec<SourceDiagnostic>>,
    errors: Vec<EcoVec<SourceDiagnostic>>,
}

impl TypstDocument {
    fn expect_paged(&self) -> Result<&Arc<TypstPagedDocument>, JsValue> {
        self.paged
            .as_ref()
            .and_then(|doc| doc.as_ref())
            .ok_or_else(|| JsError::new("Paged document not found.").into())
    }

    fn expect_html(&self) -> Result<&Arc<TypstHtmlDocument>, JsValue> {
        self.html
            .as_ref()
            .and_then(|doc| doc.as_ref())
            .ok_or_else(|| JsError::new("Html document not found.").into())
    }
}

// Convert the input pairs to a dictionary.
fn convert_inputs(inputs: &[js_sys::Array]) -> reflexo_typst::foundations::Dict {
    inputs
        .iter()
        .map(|j| {
            (
                j.get(0).as_string().unwrap_or_default().into(),
                j.get(1).as_string().into_value(),
            )
        })
        .collect()
}

pub enum JsBorrow<T: 'static> {
    Owned(T),
    Borrowed(wasm_bindgen::__rt::RcRef<T>),
}

impl<T: 'static> AsRef<T> for JsBorrow<T> {
    fn as_ref(&self) -> &T {
        match self {
            JsBorrow::Owned(t) => t,
            JsBorrow::Borrowed(t) => t,
        }
    }
}

impl<T: 'static> Deref for JsBorrow<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            JsBorrow::Owned(t) => t,
            JsBorrow::Borrowed(t) => t,
        }
    }
}

impl<T: 'static + Clone> JsBorrow<T> {
    fn to_owned(&self) -> T {
        match self {
            JsBorrow::Owned(t) => t.clone(),
            JsBorrow::Borrowed(t) => t.deref().clone(),
        }
    }
}

impl<T: Clone + IntoWasmAbi<Abi = <JsValue as FromWasmAbi>::Abi> + 'static> From<JsBorrow<T>>
    for JsValue
{
    fn from(val: JsBorrow<T>) -> Self {
        let t = match val {
            JsBorrow::Owned(t) => t,
            JsBorrow::Borrowed(t) => t.clone(),
        };

        t.into_js_value()
    }
}

pub trait JsTransmute {
    fn into_js_value(self) -> JsValue
    where
        Self: IntoWasmAbi<Abi = JsValueAbi>;
    fn from_js_value(value: JsValue) -> Self
    where
        Self: FromWasmAbi<Abi = JsValueAbi>;
    fn js_value_as_ref(abi: &JsValue) -> <Self as RefFromWasmAbi>::Anchor
    where
        Self: RefFromWasmAbi<Abi = JsValueAbi>;
}

type JsValueAbi = <JsValue as FromWasmAbi>::Abi;
impl<T: Clone + 'static> JsTransmute for T {
    fn into_js_value(self) -> JsValue
    where
        Self: IntoWasmAbi<Abi = JsValueAbi>,
    {
        // Safety: JsValue is a wrapper around the underlying ABI type
        unsafe { JsValue::from_abi(self.into_abi()) }
    }

    fn from_js_value(value: JsValue) -> Self
    where
        Self: FromWasmAbi<Abi = JsValueAbi>,
    {
        // Safety: JsValue is a wrapper around the underlying ABI type
        unsafe { Self::from_abi(value.into_abi()) }
    }

    fn js_value_as_ref(abi: &JsValue) -> <Self as RefFromWasmAbi>::Anchor
    where
        Self: RefFromWasmAbi<Abi = JsValueAbi>,
    {
        // Safety: JsValue is a wrapper around the underlying ABI type
        unsafe { Self::ref_from_abi(abi.into_abi()) }
    }
}
