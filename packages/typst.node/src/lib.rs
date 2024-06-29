/// Compiler trait for NodeJS.
pub mod compiler;

/// Error handling for NodeJS.
pub mod error;

pub use compiler::*;
use error::NodeTypstCompileResult;
pub use error::{map_node_error, NodeError};

use std::{collections::HashMap, ops::Deref, path::PathBuf, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use typst_ts_compiler::{
    Compiler, DynamicLayoutCompiler, EntryManager, SystemCompilerFeat, TypstSystemWorld,
};
use typst_ts_core::{
    diag::SourceResult, error::prelude::*, Exporter, TypstAbs, TypstDatetime, TypstDocument,
    TypstWorld,
};

/// A shared typst document object.
#[napi]
#[derive(Clone)]
pub struct NodeTypstDocument(Arc<TypstDocument>);

#[napi]
impl NodeTypstDocument {
    /// Gets the number of pages in the document.
    #[napi(getter)]
    pub fn num_of_pages(&self) -> u32 {
        self.0.pages.len() as u32
    }

    /// Gets the title of the document.
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.0.title.as_ref().map(ToString::to_string)
    }

    /// Gets the authors of the document.
    #[napi(getter)]
    pub fn authors(&self) -> Option<Vec<String>> {
        let authors = self.0.author.iter();
        Some(authors.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the keywords of the document.
    #[napi(getter)]
    pub fn keywords(&self) -> Option<Vec<String>> {
        let keywords = self.0.keywords.iter();
        Some(keywords.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the unix timestamp (in nanoseconds) of the document.
    ///
    /// Note: currently typst doesn't specify the timezone of the date, and we
    /// keep stupid and doesn't add timezone info to the date.
    #[napi(getter)]
    pub fn date(&self) -> Option<i64> {
        self.0
            .date
            .custom()
            .flatten()
            .and_then(typst_datetime_to_unix_nanoseconds)
    }

    /// Determines whether the date should be automatically generated.
    ///
    /// This happens when user specifies `date: auto` in the document
    /// explicitly.
    #[napi(getter)]
    pub fn enabled_auto_date(&self) -> bool {
        self.0.date.is_auto()
    }
}

/// Converts a typst datetime to unix nanoseconds.
fn typst_datetime_to_unix_nanoseconds(datetime: TypstDatetime) -> Option<i64> {
    let year = datetime.year().unwrap_or_default();
    let month = datetime.month().unwrap_or_default() as u32;
    let day = datetime.day().unwrap_or_default() as u32;
    let hour = datetime.hour().unwrap_or_default() as u32;
    let minute = datetime.minute().unwrap_or_default() as u32;
    let second = datetime.second().unwrap_or_default() as u32;

    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)?;
    let time = chrono::NaiveTime::from_hms_opt(hour, minute, second)?;

    let datetime = chrono::NaiveDateTime::new(date, time);

    datetime.and_utc().timestamp_nanos_opt()
}

/// Options to compile a document.
///
/// If no `mainFileContent` or `mainFilePath` is specified, the compiler will
/// use the entry file specified in the constructor of `NodeCompiler`.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CompileDocumentOptions {
    /// Directly specify the main file content.
    /// Exclusive with `mainFilePath`.
    #[serde(rename = "mainFileContent")]
    pub main_file_content: Option<String>,

    /// Path to the entry file.
    /// Exclusive with `mainFileContent`.
    #[serde(rename = "mainFilePath")]
    pub main_file_path: Option<String>,

    /// Add a string key-value pair visible through `sys.inputs`.
    pub inputs: Option<HashMap<String, String>>,
}

type MayCompileOpts<'a> = Either<&'a NodeTypstDocument, CompileDocumentOptions>;

/// Node wrapper to access compiler interfaces.
#[napi]
pub struct NodeCompiler {
    /// Inner compiler.
    driver: JsBoxedCompiler,
}

#[napi]
impl NodeCompiler {
    /// Gets default compile arguments
    #[napi]
    pub fn default_compile_args() -> NodeCompileArgs {
        NodeCompileArgs::default()
    }

    /// Creates a new compiler based on the given arguments.
    ///
    /// == Example
    ///
    /// Creates a new compiler with default arguments:
    /// ```ts
    /// const compiler = NodeCompiler.create(NodeCompiler.defaultCompileArgs());
    /// ```
    ///
    /// Creates a new compiler with custom arguments:
    /// ```ts
    /// const compiler = NodeCompiler.create({
    ///   ...NodeCompiler.defaultCompileArgs(),
    ///   workspace: '/path/to/workspace',
    ///   entry: '/path/to/entry',
    /// });
    /// ```
    #[napi]
    pub fn create(args: NodeCompileArgs) -> Result<NodeCompiler, NodeError> {
        let driver = create_driver(args).map_err(map_node_error)?;
        Ok(NodeCompiler {
            driver: driver.into(),
        })
    }

    /// Casts the inner compiler.
    #[napi]
    pub fn from_boxed(b: &mut JsBoxedCompiler) -> Self {
        NodeCompiler {
            driver: b.grab().into(),
        }
    }

    /// Takes ownership of the inner compiler.
    #[napi]
    pub fn into_boxed(&mut self) -> Result<JsBoxedCompiler, NodeError> {
        Ok(self.driver.grab().into())
    }

    /// Gets the inner world.
    fn spawn_world(&self) -> TypstSystemWorld {
        self.driver.assert_ref().deref().snapshot()
    }

    /// Compiles the document internally.
    fn compile_raw(
        &mut self,
        opts: CompileDocumentOptions,
    ) -> Result<NodeTypstCompileResult, NodeError> {
        self.driver.assert_mut().compile_raw(opts)
    }

    /// Exports the document as a specific type.
    pub fn export_as<T, O>(&mut self, doc: NodeTypstDocument) -> Result<O, NodeError>
    where
        T: Exporter<TypstDocument, O> + Default,
    {
        let e = T::default();
        e.export(&self.spawn_world(), doc.0.clone())
            .map_err(map_node_error)
    }

    /// Compiles the document as a specific type.
    pub fn compile_as<T, O>(&mut self, opts: MayCompileOpts) -> Result<O, NodeError>
    where
        T: Exporter<TypstDocument, O> + Default,
    {
        let doc = match opts {
            MayCompileOpts::A(doc) => doc.clone(),
            MayCompileOpts::B(compile_by) => {
                let mut res = self.compile_raw(compile_by)?;
                if let Some(diagnostics) = res.take_diagnostics() {
                    // todo: format diagnostics
                    return Err(Error::from_status(diagnostics));
                }

                res.result().unwrap()
            }
        };

        self.export_as::<T, O>(doc)
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
    pub fn clear_cache(&self, max_age: u32) {
        comemo::evict(usize::try_from(max_age).unwrap())
    }

    /// Compiles the document.
    #[napi]
    pub fn compile(
        &mut self,
        opts: CompileDocumentOptions,
    ) -> Result<NodeTypstCompileResult, NodeError> {
        self.compile_raw(opts)
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
    #[napi]
    pub fn query(
        &mut self,
        doc: &NodeTypstDocument,
        selector: String,
    ) -> Result<serde_json::Value, NodeError> {
        let compiler = self.driver.assert_mut();
        let world = compiler.snapshot();
        let res = compiler
            .query(&world, selector, &doc.0)
            .map_err(map_node_error)?;

        serde_json::to_value(res)
            .context("failed to serialize query result to JSON")
            .map_err(map_node_error)
    }

    /// Simply compiles the document as a vector IR.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocumentOptions")]
    pub fn vector(&mut self, compiled_or_by: MayCompileOpts) -> Result<Buffer, NodeError> {
        self.compile_as::<typst_ts_svg_exporter::SvgModuleExporter, _>(compiled_or_by)
            .map(From::from)
    }

    /// Simply compiles the document as a PDF.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocumentOptions")]
    #[cfg(feature = "pdf")]
    pub fn pdf(&mut self, compiled_or_by: MayCompileOpts) -> Result<Buffer, NodeError> {
        self.compile_as::<typst_ts_pdf_exporter::PdfDocExporter, _>(compiled_or_by)
            .map(From::from)
    }

    /// Simply compiles the document as a plain SVG.
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocumentOptions")]
    #[cfg(feature = "svg")]
    pub fn plain_svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        self.compile_as::<PlainSvgExporter, _>(compiled_or_by)
    }

    /// Simply compiles the document as a rich-contented SVG (for browsers).
    #[napi(ts_args_type = "compiledOrBy: NodeTypstDocument | CompileDocumentOptions")]
    #[cfg(feature = "svg")]
    pub fn svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        self.compile_as::<typst_ts_svg_exporter::PureSvgExporter, _>(compiled_or_by)
    }
}

#[napi]
pub struct DynLayoutCompiler {
    /// Inner compiler.
    driver: DynamicLayoutCompiler<SystemCompilerFeat, BoxedCompiler>,
}

#[napi]
impl DynLayoutCompiler {
    /// Creates a new compiler based on the given arguments.
    #[napi]
    pub fn from_boxed(b: &mut JsBoxedCompiler) -> Self {
        DynLayoutCompiler {
            driver: DynamicLayoutCompiler::new(b.grab(), PathBuf::default()),
        }
    }

    /// Specifies width (in pts) of the layout.
    pub fn set_layout_widths(&mut self, target: Vec<f64>) {
        self.driver
            .set_layout_widths(target.into_iter().map(TypstAbs::raw).collect());
    }

    /// Exports the document as a vector IR containing multiple layouts.
    #[napi]
    pub fn vector(&mut self, compile_by: CompileDocumentOptions) -> Result<Buffer, NodeError> {
        let compiler = self.driver.inner_mut();
        let world = compiler.snapshot();
        let e = compiler.setup_compiler_by(compile_by)?;
        let doc = self
            .driver
            .do_export(&world, &mut Default::default())
            .map_err(map_node_error);

        if let Some(e) = e {
            self.driver
                .inner_mut()
                .universe_mut()
                .mutate_entry(e)
                .map_err(map_node_error)?;
        }

        Ok(doc?.1.to_bytes().into())
    }
}

#[derive(Default)]
struct PlainSvgExporter {}

impl Exporter<TypstDocument, String> for PlainSvgExporter {
    fn export(&self, _world: &dyn TypstWorld, output: Arc<TypstDocument>) -> SourceResult<String> {
        Ok(typst_svg::svg_merged(&output, Default::default()))
    }
}
