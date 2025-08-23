use std::path::{Path, PathBuf};
use std::sync::Arc;

use reflexo_typst::error::prelude::*;
use reflexo_typst::program_meta::REPORT_BUG_MESSAGE;
use reflexo_typst::svg::DefaultExportFeature;
use reflexo_typst::task::{ExportHtmlTask, ExportPdfTask, ExportTextTask};
use reflexo_typst::{
    AstExport, Bytes, CompilationTask, CompileReport, ConfigTask, DiagnosticHandler,
    DiagnosticsTask, DynSvgModuleExport, DynSystemComputation, ExportAstTask, ExportComputation,
    ExportDynSvgModuleTask, ExportWebSvgHtmlTask, ExportWebSvgModuleTask, ExportWebSvgTask,
    FlagTask, HtmlCompilationTask, HtmlExport, OptionDocumentTask, PagedCompilationTask, PdfExport,
    SystemCompilerFeat, TakeAs, TextExport, WebSvgExport, WebSvgHtmlExport, WebSvgModuleExport,
    WorldComputable, WorldComputeGraph,
};
use typst::World;

use crate::{utils::current_dir, CompileArgs};

/// builtin formats should be enabled by default, and non-builtin formats should
/// be
pub static AVAILABLE_FORMATS: &[(/* format name */ &str, /* feature hint */ &str)] = &[
    ("ast", REPORT_BUG_MESSAGE),
    ("nothing", REPORT_BUG_MESSAGE),
    ("pdf", "pdf"),
    ("svg", "svg"),
    ("svg_html", "svg"),
    ("sir", "svg"),
    ("vector", "svg"),
    ("text", "text"),
];

/// Hint the user that the given format is not enable or not available.
/// Then exit the program.
fn exit_by_unknown_format(f: &str) -> ! {
    // find the feature hint
    match AVAILABLE_FORMATS.iter().find(|(k, _)| **k == *f) {
        // feat is a bug
        Some((_, feat @ REPORT_BUG_MESSAGE)) => {
            panic!("feature not enabled for format {f:?}: {feat}")
        }
        // feat is a feature hint
        Some((_, feat)) => {
            clap::Error::raw(clap::error::ErrorKind::InvalidValue,
                format!(
                    r#"feature not enabled for format {f:?}: suggested feature "{feat}". To figure out enabled features, use command "$program env features"
"#
                )).exit()
        }
        // f is an unknown format
        None => {
            clap::Error::raw(clap::error::ErrorKind::UnknownArgument,
                format!(
                    "unknown format: {f}\n"
                )).exit()
        }
    }
}

#[derive(Clone)]
pub enum ReflexoTask {
    Ast(ExportAstTask),
    Pdf(ExportPdfTask),
    Html(ExportHtmlTask),
    WebSvg(ExportWebSvgTask),
    WebSvgHtml(ExportWebSvgHtmlTask),
    WebSvgModule(ExportWebSvgModuleTask),
    DynSvgModule(ExportDynSvgModuleTask),
    Text(ExportTextTask),
}

#[derive(Default, Clone)]
pub struct ReflexoTaskBuilder {
    diag_handler: DiagnosticHandler,
    output_path: PathBuf,
    tasks: Vec<ReflexoTask>,
}

impl ReflexoTaskBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn args(&mut self, args: &CompileArgs, entry_file: Option<&Path>) -> &mut Self {
        self.diag_handler = args.diagnostics_handler();

        self.output_path = {
            // If output is specified, use it.
            let dir = (!args.compile.output.is_empty()).then(|| Path::new(&args.compile.output));
            // Otherwise, use the parent directory of the entry file.
            let dir = dir.map(Path::to_owned).unwrap_or_else(|| match entry_file {
                Some(entry_file) => entry_file
                    .parent()
                    .expect("entry_file has no parent")
                    .to_owned(),
                None => current_dir(),
            });
            match entry_file {
                Some(entry_file) => {
                    dir.join(entry_file.file_name().expect("entry_file has no file name"))
                }
                None => dir.join("main"),
            }
        };
        let mut formats = {
            // If formats are specified, use them.
            let mut formats = args.format.clone();
            // Otherwise, use default formats.
            if formats.is_empty() {
                formats.extend(["pdf", "vector"].map(str::to_owned));
            }
            formats
        };
        formats.sort();
        formats.dedup();

        for format in formats.iter() {
            match format.as_str() {
                "nothing" => {}
                #[cfg(feature = "ast")]
                "ast" => {
                    self.add_ast(ExportAstTask);
                }
                #[cfg(feature = "pdf")]
                "pdf" => {
                    self.add_pdf(ExportPdfTask {
                        creation_timestamp: args.export.creation_timestamp,
                        ..ExportPdfTask::default()
                    });
                }
                #[cfg(feature = "html")]
                "html" => {
                    self.add_html(ExportHtmlTask::default());
                }
                #[cfg(feature = "svg")]
                "svg" => {
                    self.add_web_svg(ExportWebSvgTask::default());
                }
                #[cfg(feature = "svg")]
                "svg_html" => {
                    self.add_web_svg_html(ExportWebSvgHtmlTask::default());
                }
                #[cfg(feature = "svg")]
                "sir" | "vector" => {
                    self.add_web_svg_module(ExportWebSvgModuleTask::default());
                }
                #[cfg(feature = "svg")]
                "text" => {
                    self.add_text(ExportTextTask::default());
                }
                format => exit_by_unknown_format(format),
            }
        }

        // todo: dynamic layout of other formats
        if args.dynamic_layout {
            self.add_dyn_svg_module(ExportDynSvgModuleTask::default());
        }

        self
    }

    pub fn add_ast(&mut self, config: ExportAstTask) -> &mut Self {
        self.tasks.push(ReflexoTask::Ast(config));
        self
    }

    pub fn add_pdf(&mut self, config: ExportPdfTask) -> &mut Self {
        self.tasks.push(ReflexoTask::Pdf(config));
        self
    }

    pub fn add_html(&mut self, config: ExportHtmlTask) -> &mut Self {
        self.tasks.push(ReflexoTask::Html(config));
        self
    }

    pub fn add_web_svg(&mut self, config: ExportWebSvgTask) -> &mut Self {
        self.tasks.push(ReflexoTask::WebSvg(config));
        self
    }

    pub fn add_web_svg_html(&mut self, config: ExportWebSvgHtmlTask) -> &mut Self {
        self.tasks.push(ReflexoTask::WebSvgHtml(config));
        self
    }

    pub fn add_web_svg_module(&mut self, config: ExportWebSvgModuleTask) -> &mut Self {
        self.tasks.push(ReflexoTask::WebSvgModule(config));
        self
    }

    pub fn add_dyn_svg_module(&mut self, config: ExportDynSvgModuleTask) -> &mut Self {
        self.tasks.push(ReflexoTask::DynSvgModule(config));
        self
    }

    pub fn add_text(&mut self, config: ExportTextTask) -> &mut Self {
        self.tasks.push(ReflexoTask::Text(config));
        self
    }

    pub fn build(self) -> DynSystemComputation {
        prepare_exporters_impl(self.diag_handler, self.output_path, self.tasks)
    }

    pub fn set_output_path(&mut self, output_path: PathBuf) {
        self.output_path = output_path;
    }

    pub fn print_compile_status(&mut self, enabled: bool) -> &mut Self {
        self.diag_handler.print_compile_status = enabled;
        self
    }
}

/// With the given arguments, prepare exporters for the compilation.
fn prepare_exporters_impl(
    diag_handler: DiagnosticHandler,
    out: PathBuf,
    tasks: Vec<ReflexoTask>,
) -> DynSystemComputation {
    type EF = DefaultExportFeature;

    fn export_to_path(result: Result<Option<Bytes>>, output_path: PathBuf) {
        let result = match result {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return,
            Err(err) => {
                eprintln!("export failed: {err}");
                return;
            }
        };

        let err = std::fs::write(output_path, result.as_slice());
        if let Err(err) = err {
            eprintln!("export failed: {err}");
        }
    }

    fn compile_it<D: typst::Document + Send + Sync + 'static>(
        graph: &Arc<WorldComputeGraph<SystemCompilerFeat>>,
    ) -> Result<Option<Arc<D>>> {
        let _ = graph.provide::<FlagTask<CompilationTask<D>>>(Ok(FlagTask::flag(true)));
        graph.compute::<OptionDocumentTask<D>>().map(Arc::take)
    }

    fn export_bytes<
        D: typst::Document + Send + Sync + 'static,
        T: ExportComputation<SystemCompilerFeat, D, Output = Bytes>,
    >(
        graph: &Arc<WorldComputeGraph<SystemCompilerFeat>>,
        config: &T::Config,
    ) -> Result<Option<Bytes>> {
        let doc = compile_it::<D>(graph)?;

        let res = doc.as_ref().map(|doc| T::run(graph, doc, config));
        res.transpose()
    }

    fn export_string<
        D: typst::Document + Send + Sync + 'static,
        T: ExportComputation<SystemCompilerFeat, D, Output = String>,
    >(
        graph: &Arc<WorldComputeGraph<SystemCompilerFeat>>,
        config: &T::Config,
    ) -> Result<Option<Bytes>> {
        let doc = compile_it::<D>(graph)?;

        let doc = doc.as_ref();
        let res = doc.map(|doc| T::run(graph, doc, config).map(Bytes::from_string));
        res.transpose()
    }

    Arc::new(move |graph: &Arc<WorldComputeGraph<SystemCompilerFeat>>| {
        let start = reflexo_typst::time::now();
        let main = graph.snap.world.main();

        diag_handler.status(&CompileReport::Stage(main, "compiling", start));

        for task in tasks.iter() {
            use ReflexoTask::*;
            match task {
                #[cfg(feature = "ast")]
                Ast(_config) => {
                    let output_path = out.with_extension("ast.ansi.text");
                    let result = AstExport::compute(graph);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "pdf")]
                Pdf(config) => {
                    let output_path = out.with_extension("pdf");
                    let result = export_bytes::<_, PdfExport>(graph, config);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "html")]
                Html(config) => {
                    let output_path = out.with_extension("html");
                    let result = export_string::<_, HtmlExport>(graph, config);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "svg")]
                WebSvg(config) => {
                    let output_path = out.with_extension("artifact.svg");
                    let result = export_string::<_, WebSvgExport<EF>>(graph, config);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "svg")]
                WebSvgHtml(config) => {
                    let output_path = out.with_extension("artifact.svg.html");
                    let result = export_string::<_, WebSvgHtmlExport<EF>>(graph, config);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "svg")]
                WebSvgModule(config) => {
                    let output_path = out.with_extension("artifact.sir.in");
                    let result = export_bytes::<_, WebSvgModuleExport<EF>>(graph, config);
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "svg")]
                DynSvgModule(config) => {
                    let output_path = out.with_extension("multi.sir.in");
                    let result = DynSvgModuleExport::run(graph, config);
                    let result = result.map(|d| d.map(|d| Bytes::new(d.to_bytes())));
                    export_to_path(result, output_path);
                }
                #[cfg(feature = "text")]
                Text(config) => {
                    let output_path = out.with_extension("txt");
                    let result = export_string::<_, TextExport>(graph, config);
                    export_to_path(result, output_path);
                }
            }
        }

        let _ = graph.provide::<FlagTask<PagedCompilationTask>>(Ok(FlagTask::flag(false)));
        let _ = graph.provide::<FlagTask<HtmlCompilationTask>>(Ok(FlagTask::flag(false)));

        let diag = graph.compute::<DiagnosticsTask>()?;

        let error_cnt = diag.error_cnt();
        let warning_cnt = diag.warning_cnt();

        let report = if error_cnt != 0 {
            CompileReport::CompileError(main, error_cnt, start.elapsed().unwrap_or_default())
        } else {
            CompileReport::CompileSuccess(main, warning_cnt, start.elapsed().unwrap_or_default())
        };

        diag_handler.status(&report);
        let _ = graph.provide::<ConfigTask<CompileReport>>(Ok(Arc::new(report)));

        // todo: export diagnostics.
        diag_handler.report(&graph.snap.world, diag.diagnostics());

        Ok(())
    })
}

/// Prepare exporters from command line arguments.
pub fn prepare_exporters(args: &CompileArgs, entry_file: Option<&Path>) -> DynSystemComputation {
    let mut tb = ReflexoTaskBuilder::new();
    tb.args(args, entry_file);
    tb.build()
}
