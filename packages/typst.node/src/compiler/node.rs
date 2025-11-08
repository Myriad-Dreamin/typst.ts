use std::sync::Arc;
use std::{ops::Deref, path::Path};

use napi_derive::napi;
use reflexo_typst::syntax::Span;
use reflexo_typst::typst::diag::At;
use reflexo_typst::{error::WithContext, DocumentQuery, ExportComputation, ExportWebSvgModuleTask};
use reflexo_typst::{
    ArcInto, Bytes, ExportDynSvgModuleTask, ShadowApi, SystemCompilerFeat, TypstAbs, TypstDocument,
    TypstDocumentTrait, TypstPagedDocument, TypstSystemWorld,
};
use tinymist_project::ImageOutput;

use crate::error::*;
use crate::{
    create_universe, BoxedCompiler, Buffer, CompileArgs, CompileDocArgs, Either, Error,
    JsBoxedCompiler, NodeTypstDocument, QueryDocArgs, RenderPdfOpts, Result,
};

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
    #[napi(ts_args_type = "args?: CompileArgs")]
    pub fn create(args: Option<CompileArgs>) -> Result<NodeCompiler, NodeError> {
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

    /// Compiles the document internally.
    fn compile_raw<D: TypstDocumentTrait + ArcInto<TypstDocument> + Send + Sync + 'static>(
        &mut self,
        opts: CompileDocArgs,
    ) -> Result<NodeTypstCompileResult, NodeError> {
        let result = self.driver.assert_mut().compile_raw2::<D>(opts);
        Ok(result.map_err(map_node_error)?.into())
    }

    /// Compiles the document internally.
    fn compile_raw2<D: TypstDocumentTrait + ArcInto<TypstDocument> + Send + Sync + 'static>(
        &mut self,
        opts: CompileDocArgs,
    ) -> std::result::Result<ExecResultRepr<NodeTypstDocument>, NodeError> {
        self.driver.assert_mut().compile_raw2::<D>(opts)
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

            let pdf_tags = opts.pdf_tags.unwrap_or(true);

            ExportPdfTask {
                export: Default::default(),
                pdf_standards: standard.into_iter().collect(),
                no_pdf_tags: !pdf_tags,
                creation_timestamp,
                pages: None,
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
        let output = self
            .compile_as::<Export, ImageOutput<String>>(compiled_or_by, &ExportSvgTask::default())?;
        match output {
            ImageOutput::Merged(s) => Ok(s),
            ImageOutput::Paged(..) => unreachable!(),
        }
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
        let graph = self
            .driver
            .computation(compile_by)
            .map_err(map_node_error)?;
        let world = &graph.snap.world;

        let doc = self.task.do_export(world).map_err(map_node_error)?;

        Ok(doc.to_bytes().into())
    }
}
