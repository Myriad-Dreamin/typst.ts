use boxed::{BoxedCompiler, NodeCompilerTrait};
use core::fmt;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::{
    cell::OnceCell,
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use typst_ts_compiler::{
    font::system::SystemFontSearcher,
    package::http::HttpRegistry,
    service::{CompileDriver, CompileMiddleware, Compiler, DynamicLayoutCompiler},
    vfs::{system::SystemAccessModel, Vfs},
    TypstSystemWorld,
};
use typst_ts_core::{
    config::CompileOpts,
    error::{prelude::*, TypstSourceDiagnostic},
    typst::{foundations::IntoValue, prelude::*},
    Bytes, Exporter, TypstDict,
};

pub mod boxed;

impl NodeCompilerTrait for CompileDriver {}

// A complex struct which cannot be exposed to JavaScript directly.
#[napi(js_name = "BoxedCompiler")]
pub struct JsBoxedCompiler {
    inner: Option<BoxedCompiler>,
}

impl JsBoxedCompiler {
    fn grab(&mut self) -> BoxedCompiler {
        self.inner.take().expect("moved box compiler")
    }
}

pub enum NodeErrorStatus {
    Error(typst_ts_core::error::Error),
    Diagnostics(EcoVec<TypstSourceDiagnostic>),
}

impl fmt::Display for NodeErrorStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeErrorStatus::Error(e) => write!(f, "{}", e),
            NodeErrorStatus::Diagnostics(diagnostics) => {
                for diagnostic in diagnostics {
                    writeln!(f, "{}\n", diagnostic.message)?;
                }
                Ok(())
            }
        }
    }
}

pub struct NodeError(OnceCell<String>, NodeErrorStatus);

impl From<typst_ts_core::error::Error> for NodeError {
    fn from(e: typst_ts_core::error::Error) -> Self {
        NodeError(OnceCell::new(), NodeErrorStatus::Error(e))
    }
}

impl From<EcoVec<TypstSourceDiagnostic>> for NodeError {
    fn from(e: EcoVec<TypstSourceDiagnostic>) -> Self {
        NodeError(OnceCell::new(), NodeErrorStatus::Diagnostics(e))
    }
}

impl AsRef<str> for NodeError {
    fn as_ref(&self) -> &str {
        self.0.get_or_init(|| self.1.to_string())
    }
}

// |e| napi::Error::from_status(NodeError::new(e))
fn map_node_error(e: impl Into<NodeError>) -> napi::Error<NodeError> {
    napi::Error::from_status(e.into())
}

#[napi]
pub struct NodeTypstDocument(Arc<typst_ts_core::typst::TypstDocument>);

#[napi]
impl NodeTypstDocument {
    #[napi(getter)]
    pub fn title(&self) -> napi::Result<Option<String>> {
        Ok(self.0.title.as_ref().map(ToString::to_string))
    }
}

#[napi(object)]
pub struct NodeAddFontPaths {
    /// Add additional directories to search for fonts
    pub font_paths: Vec<String>,
}

#[napi(object)]
pub struct NodeAddFontBlobs {
    /// Add additional memory fonts
    pub font_blobs: Vec<Vec<u8>>,
}

#[napi(object, js_name = "CompileArgs")]
#[derive(Default)]
pub struct NodeCompileArgs {
    /// Add additional directories to search for fonts
    pub font_args: Vec<Either<NodeAddFontPaths, NodeAddFontBlobs>>,

    /// Path to typst workspace.
    pub workspace: String,

    /// Entry file.
    pub entry: String,

    /// Add a string key-value pair visible through `sys.inputs`
    pub inputs: HashMap<String, String>,
}

fn create_driver(args: NodeCompileArgs) -> ZResult<CompileDriver> {
    use typst_ts_core::path::PathClean;
    let workspace_dir = Path::new(args.workspace.as_str()).clean();
    let entry_file_path = Path::new(args.entry.as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(workspace_dir)
    };

    let entry_file_path = if entry_file_path.is_absolute() {
        entry_file_path
    } else {
        let cwd = std::env::current_dir().context("failed to get current dir")?;
        cwd.join(entry_file_path)
    };

    let workspace_dir = workspace_dir.clean();
    let entry_file_path = entry_file_path.clean();

    if !entry_file_path.starts_with(&workspace_dir) {
        return Err(error_once!(
            "entry file path must be in workspace directory",
            workspace_dir: workspace_dir.display()
        ));
    }

    // Convert the input pairs to a dictionary.
    let inputs: TypstDict = args
        .inputs
        .iter()
        .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
        .collect();

    let mut searcher = SystemFontSearcher::new();

    for arg in args.font_args {
        match arg {
            Either::A(p) => {
                for i in p.font_paths {
                    let path = Path::new(&i);
                    if path.is_dir() {
                        searcher.search_dir(path);
                    } else {
                        let _ = searcher.search_file(path);
                    }
                }
            }
            Either::B(p) => {
                for b in p.font_blobs {
                    searcher.add_memory_font(Bytes::from(b));
                }
            }
        }
    }

    searcher.resolve_opts(CompileOpts {
        with_embedded_fonts: typst_ts_cli::font::EMBEDDED_FONT.to_owned(),
        ..CompileOpts::default()
    })?;

    let mut world = TypstSystemWorld::new_raw(
        workspace_dir.clone(),
        Vfs::new(SystemAccessModel {}),
        HttpRegistry::default(),
        searcher.into(),
    );
    world.set_inputs(Arc::new(Prehashed::new(inputs)));

    Ok(CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
    })
}

/// `constructor` option for `struct` requires all fields to be public,
/// otherwise tag impl fn as constructor
/// #[napi(constructor)]
#[napi]
pub struct NodeCompiler {
    driver: Option<CompileDriver>,
}

#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CompileBy {
    #[serde(rename = "mainFileContent")]
    pub main_file_content: Option<String>,
    #[serde(rename = "mainFilePath")]
    pub main_file_path: Option<String>,
}

#[napi]
impl NodeCompiler {
    /// Get default compile arguments
    #[napi]
    pub fn default_compile_args() -> NodeCompileArgs {
        NodeCompileArgs::default()
    }

    /// This is the constructor
    #[napi]
    pub fn create(args: NodeCompileArgs) -> Result<NodeCompiler, NodeError> {
        let driver = create_driver(args).map_err(map_node_error)?;
        Ok(NodeCompiler {
            driver: Some(driver),
        })
    }

    fn compile_raw(&mut self, compile_by: CompileBy) -> Result<NodeTypstDocument, NodeError> {
        let t = self.driver.as_mut().expect("moved box compiler");
        t.compile_raw(compile_by)
    }

    #[napi]
    pub fn compile(&mut self, compile_by: CompileBy) -> Result<NodeTypstDocument, NodeError> {
        self.compile_raw(compile_by)
    }

    fn world(&self) -> &TypstSystemWorld {
        self.driver.as_ref().expect("moved compiler").world()
    }

    #[napi]
    pub fn vector(&mut self, compile_by: CompileBy) -> Result<Buffer, NodeError> {
        let res = self.compile_raw(compile_by)?;

        let e = typst_ts_svg_exporter::SvgModuleExporter::default();
        let res = e.export(self.world(), res.0).map_err(map_node_error)?;

        Ok(res.into())
    }

    #[napi]
    #[cfg(feature = "pdf")]
    pub fn pdf(&mut self, compile_by: CompileBy) -> Result<Buffer, NodeError> {
        let res = self.compile_raw(compile_by)?;

        let e = typst_ts_pdf_exporter::PdfDocExporter::default();
        let res = e.export(self.world(), res.0).map_err(map_node_error)?;

        Ok(res.into())
    }

    #[napi]
    #[cfg(feature = "svg")]
    pub fn svg(&mut self, compile_by: CompileBy) -> Result<String, NodeError> {
        let res = self.compile_raw(compile_by)?;

        let e = typst_ts_svg_exporter::PureSvgExporter;
        e.export(self.world(), res.0).map_err(map_node_error)
    }

    #[napi]
    pub fn into_boxed(&mut self) -> Result<JsBoxedCompiler, NodeError> {
        let driver = self.driver.take().expect("moved compiler");
        Ok(JsBoxedCompiler {
            inner: Some(driver.into()),
        })
    }
}

#[napi]
pub struct DynLayoutCompiler {
    driver: DynamicLayoutCompiler<BoxedCompiler>,
}

#[napi]
impl DynLayoutCompiler {
    #[napi]
    pub fn from_boxed(b: &mut JsBoxedCompiler) -> Self {
        DynLayoutCompiler {
            driver: DynamicLayoutCompiler::new(b.grab(), PathBuf::default()),
        }
    }

    #[napi]
    pub fn vector(&mut self, compile_by: CompileBy) -> Result<Buffer, NodeError> {
        let e = self.driver.inner_mut().setup_compiler_by(compile_by)?;
        let doc = self.driver.do_export().map_err(map_node_error);

        if let Some(e) = e {
            self.driver
                .inner_mut()
                .set_entry_file(e)
                .map_err(map_node_error)?;
        }

        Ok(doc?.to_bytes().into())
    }
}
