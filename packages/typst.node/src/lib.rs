/// Compiler trait for NodeJS.
pub mod compiler;

/// Error handling for NodeJS.
pub mod error;

pub use compiler::*;
pub use error::{map_node_error, NodeError};

use std::{collections::HashMap, ops::Deref, path::Path, sync::Arc};

use napi::bindgen_prelude::*;
use napi_derive::napi;
use reflexo_typst::syntax::Span;
use reflexo_typst::typst::diag::At;
use reflexo_typst::{
    error::WithContext, DocumentQuery, ExportComputation, ExportWebSvgModuleTask, WorldComputeGraph,
};
use reflexo_typst::{
    Bytes, ExportDynSvgModuleTask, ShadowApi, SystemCompilerFeat, TypstAbs, TypstDatetime,
    TypstPagedDocument, TypstSystemWorld,
};
use serde::{Deserialize, Serialize};

use error::NodeTypstCompileResult;

/// A shared typst document object.
#[napi]
#[derive(Clone)]
pub struct NodeTypstDocument {
    /// The cache of exports.
    pub(crate) graph: Arc<WorldComputeGraph<SystemCompilerFeat>>,
    /// Inner document.
    pub(crate) doc: Arc<TypstPagedDocument>,
}

#[napi]
impl NodeTypstDocument {
    /// Gets the number of pages in the document.
    #[napi(getter)]
    pub fn num_of_pages(&self) -> u32 {
        self.doc.pages.len() as u32
    }

    /// Gets the title of the document.
    #[napi(getter)]
    pub fn title(&self) -> Option<String> {
        self.doc.info.title.as_ref().map(ToString::to_string)
    }

    /// Gets the authors of the document.
    #[napi(getter)]
    pub fn authors(&self) -> Option<Vec<String>> {
        let authors = self.doc.info.author.iter();
        Some(authors.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the keywords of the document.
    #[napi(getter)]
    pub fn keywords(&self) -> Option<Vec<String>> {
        let keywords = self.doc.info.keywords.iter();
        Some(keywords.map(ToString::to_string).collect::<Vec<_>>())
    }

    /// Gets the unix timestamp (in nanoseconds) of the document.
    ///
    /// Note: currently typst doesn't specify the timezone of the date, and we
    /// keep stupid and doesn't add timezone info to the date.
    #[napi(getter)]
    pub fn date(&self) -> Option<i64> {
        self.doc
            .info
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
        self.doc.info.date.is_auto()
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

/// Arguments to compile a document.
///
/// If no `mainFileContent` or `mainFilePath` is specified, the compiler will
/// use the entry file specified in the constructor of `NodeCompiler`.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct CompileDocArgs {
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

/// Arguments to query the document.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDocArgs {
    /// The query selector.
    pub selector: String,
    /// An optional field to select on the element of the resultants.
    pub field: Option<String>,
}

/// Arguments to render a PDF.
#[napi(object)]
#[derive(Serialize, Deserialize, Debug)]
#[cfg(feature = "pdf")]
pub struct RenderPdfOpts {
    /// (Experimental) An optional PDF standard to be used to export PDF.
    ///
    /// Please check {@link types.PdfStandard} for a non-exhaustive list of
    /// standards.
    pub pdf_standard: Option<String>,

    /// An optional (creation) timestamp to be used to export PDF.
    ///
    /// This is used when you *enable auto timestamp* in the document.
    pub creation_timestamp: Option<i64>,
}

/// Either a compiled document or compile arguments.
type MayCompileOpts<'a> = Either<&'a NodeTypstDocument, CompileDocArgs>;

/// Node wrapper to access compiler interfaces.
#[napi]
pub struct NodeCompiler {
    /// Inner compiler.
    driver: JsBoxedCompiler,
}

#[napi]
impl NodeCompiler {
    /// Creates a new compiler based on the given arguments.
    ///
    /// == Example
    ///
    /// Creates a new compiler with default arguments:
    /// ```ts
    /// const compiler = NodeCompiler.create();
    /// ```
    ///
    /// Creates a new compiler with custom arguments:
    /// ```ts
    /// const compiler = NodeCompiler.create({
    ///   workspace: '/path/to/workspace',
    /// });
    /// ```
    #[napi]
    pub fn create(args: Option<NodeCompileArgs>) -> Result<NodeCompiler, NodeError> {
        let driver = create_universe(args).map_err(map_node_error)?;
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
    pub fn evict_cache(&mut self, max_age: u32) {
        let max_age = usize::try_from(max_age).unwrap();
        comemo::evict(max_age);
        self.driver.assert_mut().evict(max_age);
    }

    /// Adds a source file to the compiler.
    /// @param path - The path of the source file.
    /// @param source - The source code of the source file.
    #[napi]
    pub fn add_source(&mut self, path: String, source: String) -> Result<(), NodeError> {
        let content = Bytes::new(source.into_bytes());
        let verse = self.driver.assert_mut();
        let res = verse.map_shadow(Path::new(&path), content);
        res.at(Span::detached()).map_err(map_node_error)
    }

    /// Adds a shadow file to the compiler.
    /// @param path - The path to the shadow file.
    /// @param content - The content of the shadow file.
    #[napi]
    pub fn map_shadow(&mut self, path: String, content: Buffer) -> Result<(), NodeError> {
        let content = Bytes::new(content.as_ref().to_vec());
        let verse = self.driver.assert_mut();
        let res = verse.map_shadow(Path::new(&path), content);
        res.at(Span::detached()).map_err(map_node_error)
    }

    /// Removes a shadow file from the compiler.
    /// @param path - The path to the shadow file.
    #[napi]
    pub fn unmap_shadow(&mut self, path: String) -> Result<(), NodeError> {
        let verse = self.driver.assert_mut();
        let res = verse.unmap_shadow(Path::new(&path));
        res.at(Span::detached()).map_err(map_node_error)
    }

    /// Resets the shadow files.
    /// Note: this function is independent to the {@link reset} function.
    #[napi]
    pub fn reset_shadow(&mut self) {
        self.driver.assert_mut().reset_shadow();
    }

    /// Compiles the document.
    #[napi]
    pub fn compile(&mut self, opts: CompileDocArgs) -> Result<NodeTypstCompileResult, NodeError> {
        self.compile_raw(opts)
    }

    /// Compiles the document internally.
    fn compile_raw(&mut self, opts: CompileDocArgs) -> Result<NodeTypstCompileResult, NodeError> {
        self.driver.assert_mut().compile_raw(opts)
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
    #[napi(
        ts_args_type = "compiledOrBy: NodeTypstPagedDocument | CompileDocArgs, args: QueryDocArgs"
    )]
    pub fn query(
        &mut self,
        opts: MayCompileOpts,
        args: QueryDocArgs,
    ) -> Result<serde_json::Value, NodeError> {
        let doc = self.may_compile(opts)?;

        let config = reflexo_typst::task::QueryTask {
            export: reflexo_typst::task::ExportTask::default(),
            format: "json".to_owned(),
            output_extension: None,
            selector: args.selector,
            field: args.field,
            one: false,
        };

        DocumentQuery::get_as_value(&doc.graph, &doc.doc, &config).map_err(map_node_error)
    }

    /// Compiles the document as a specific type.
    pub fn may_compile(&mut self, opts: MayCompileOpts) -> Result<NodeTypstDocument, NodeError> {
        Ok(match opts {
            MayCompileOpts::A(doc) => doc.clone(),
            MayCompileOpts::B(compile_by) => {
                let mut res = self.compile_raw(compile_by)?;
                if let Some(diagnostics) = res.take_diagnostics() {
                    // todo: format diagnostics
                    return Err(Error::from_status(diagnostics));
                }

                res.result().unwrap()
            }
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
        let doc = self.may_compile(opts)?;
        T::run(&doc.graph, &doc.doc, config)
            .map_err(map_node_error)
            .map(From::from)
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
    #[napi(ts_args_type = "compiledOrBy: NodeTypstPagedDocument | CompileDocArgs")]
    pub fn vector(&mut self, compiled_or_by: MayCompileOpts) -> Result<Buffer, NodeError> {
        use reflexo_vec2svg::DefaultExportFeature;
        type Export = reflexo_typst::WebSvgModuleExport<DefaultExportFeature>;
        self.compile_as_buffer::<Export>(compiled_or_by, &ExportWebSvgModuleTask::default())
    }

    /// Simply compiles the document as a PDF.
    #[napi(
        ts_args_type = "compiledOrBy: NodeTypstPagedDocument | CompileDocArgs, opts?: RenderPdfOpts"
    )]
    #[cfg(feature = "pdf")]
    pub fn pdf(
        &mut self,
        compiled_or_by: MayCompileOpts,
        opts: Option<RenderPdfOpts>,
    ) -> Result<Buffer, NodeError> {
        type Export = reflexo_typst::PdfExport;
        use reflexo_typst::task::ExportPdfTask;

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
    #[napi(ts_args_type = "compiledOrBy: NodeTypstPagedDocument | CompileDocArgs")]
    #[cfg(feature = "svg")]
    pub fn plain_svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        use reflexo_typst::task::ExportSvgTask;

        type Export = reflexo_typst::SvgExport;
        self.compile_as::<Export, _>(compiled_or_by, &ExportSvgTask::default())
    }

    /// Simply compiles the document as a rich-contented SVG (for browsers).
    #[napi(ts_args_type = "compiledOrBy: NodeTypstPagedDocument | CompileDocArgs")]
    #[cfg(feature = "svg")]
    pub fn svg(&mut self, compiled_or_by: MayCompileOpts) -> Result<String, NodeError> {
        use reflexo_typst::ExportWebSvgTask;
        use reflexo_vec2svg::DefaultExportFeature;

        type Export = reflexo_typst::WebSvgExport<DefaultExportFeature>;
        self.compile_as::<Export, _>(compiled_or_by, &ExportWebSvgTask::default())
    }
}

#[napi]
pub struct DynLayoutCompiler {
    driver: BoxedCompiler,
    task: ExportDynSvgModuleTask,
}

#[napi]
impl DynLayoutCompiler {
    /// Creates a new compiler based on the given arguments.
    #[napi]
    pub fn from_boxed(b: &mut JsBoxedCompiler) -> Self {
        DynLayoutCompiler {
            driver: b.grab(),
            task: ExportDynSvgModuleTask::default(),
        }
    }

    /// Sets the target of the compiler.
    #[napi]
    pub fn set_target(&mut self, target: String) {
        self.task.set_target(target);
    }

    /// Specifies width (in pts) of the layout.
    #[napi]
    pub fn set_layout_widths(&mut self, layout_widths: Vec<f64>) {
        self.task
            .set_layout_widths(layout_widths.into_iter().map(TypstAbs::pt).collect());
    }

    /// Exports the document as a vector IR containing multiple layouts.
    #[napi]
    pub fn vector(&mut self, compile_by: CompileDocArgs) -> Result<Buffer, NodeError> {
        let graph = self.driver.computation(compile_by)?;
        let world = &graph.snap.world;

        let doc = self.task.do_export(world).map_err(map_node_error)?;

        Ok(doc.to_bytes().into())
    }
}
