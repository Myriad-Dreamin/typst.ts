use std::path::{Path, PathBuf};

use typst_ts_core::{
    exporter_builtins::{FsPathExporter, GroupExporter},
    program_meta::REPORT_BUG_MESSAGE,
};
use typst_ts_svg_exporter::DefaultExportFeature;

use crate::{stdin_path, utils::current_dir, CompileArgs, ExportArgs};

type GroupDocExporter = GroupExporter<typst::model::Document>;

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
            panic!("feature not enabled for format {:?}: {}", f, feat)
        }
        // feat is a feature hint
        Some((_, feat)) => {
            clap::Error::raw(clap::error::ErrorKind::InvalidValue,
                format!(
                    r#"feature not enabled for format {:?}: suggested feature "{}". To figure out enabled features, use command "$program env features"
"#,
                    f, feat
                )).exit()
        }
        // f is an unknown format
        None => {
            clap::Error::raw(clap::error::ErrorKind::UnknownArgument,
                format!(
                    "unknown format: {}\n", f
                )).exit()
        }
    }
}

/// With the given arguments, prepare exporters for the compilation.
fn prepare_exporters_impl(
    args: ExportArgs,
    out: PathBuf,
    mut formats: Vec<String>,
) -> GroupDocExporter {
    let mut doc: ExporterVec<Doc> = vec![];

    /// connect export flow from $x to $y
    #[allow(unused_macros)]
    macro_rules! sink_flow {
        ($x:ident -> $y:ident) => {
            if !$y.is_empty() {
                $x.push(Box::new(FromExporter::new($y)));
            }
        };
    }

    /// write $exporters as $exporter to path `$output_dir @@ $extension`
    macro_rules! sink_path {
        ($exporter:ty as $ser:ty as $exporters:ident, $output_dir:ident @@ $extension:literal) => {{
            let output_path = $output_dir.with_extension($extension);
            $exporters.push(Box::new(FsPathExporter::<$ser, _>::new(
                output_path,
                <$exporter>::default(),
            )));
        }};
        (|| $exporter:tt as $ser:ty as $exporters:ident, $output_dir:ident @@ $extension:literal) => {{
            let output_path = $output_dir.with_extension($extension);
            let exporter = $exporter;
            $exporters.push(Box::new(FsPathExporter::<$ser, _>::new(
                output_path,
                exporter,
            )));
        }};
    }

    // sink exporters according to the given formats
    {
        formats.sort();
        formats.dedup();
        #[rustfmt::skip]
        formats.iter().map(String::as_str).for_each(|f| match f {
            "nothing"     => (),
            "ast"         => sink_path!(WithAst as _ as doc, out @@ "ast.ansi.text"),
            #[cfg(feature = "pdf")]
            "pdf"         => sink_path!(|| {
                WithPdf::default().with_timestamp(args.pdf_timestamp)
            } as _ as doc, out @@ "pdf"),
            #[cfg(feature = "svg")]
            "svg"         => sink_path!(WithSvg as _ as doc, out @@ "artifact.svg"),
            #[cfg(feature = "svg")]
            "svg_html"         => sink_path!(WithSvgHtml as _ as doc, out @@ "artifact.svg.html"),
            #[cfg(feature = "svg")]
            "sir"         => sink_path!(WithSIR as _ as doc, out @@ "artifact.sir.in"),
            #[cfg(feature = "svg")]
            "vector"      => sink_path!(WithSIR as _ as doc, out @@ "artifact.sir.in"),
            #[cfg(feature = "text")]
            "text"      => sink_path!(WithText as _ as doc, out @@ "txt"),
            _             => exit_by_unknown_format(f),
        });
    }
    return GroupExporter::new(doc);

    type Doc = typst::model::Document;

    type WithAst = typst_ts_ast_exporter::AstExporter;
    // type WithJson<T> = typst_ts_serde_exporter::JsonExporter<T>;
    type WithPdf = typst_ts_pdf_exporter::PdfDocExporter;
    // type WithRmp<T> = typst_ts_serde_exporter::RmpExporter<T>;
    type WithSvg = typst_ts_svg_exporter::PureSvgExporter;
    type WithSvgHtml = typst_ts_svg_exporter::SvgExporter<DefaultExportFeature>;
    type WithSIR = typst_ts_svg_exporter::SvgModuleExporter;
    type WithText = typst_ts_text_exporter::TextExporter;

    type ExporterVec<T> = Vec<Box<dyn typst_ts_core::Exporter<T> + Send>>;
}

/// Prepare exporters from command line arguments.
pub fn prepare_exporters(args: &CompileArgs, entry_file: &Path) -> GroupDocExporter {
    let output_dir = {
        // If output is specified, use it.
        let dir = (!args.compile.output.is_empty()).then(|| Path::new(&args.compile.output));
        // Otherwise, use the parent directory of the entry file.
        let dir = dir.map(Path::to_owned).unwrap_or_else(|| {
            if entry_file == stdin_path() {
                current_dir()
            } else {
                entry_file
                    .parent()
                    .expect("entry_file has no parent")
                    .to_owned()
            }
        });
        if entry_file == stdin_path() {
            dir.join("main")
        } else {
            dir.join(entry_file.file_name().expect("entry_file has no file name"))
        }
    };

    let formats = {
        // If formats are specified, use them.
        let mut formats = args.format.clone();
        // Otherwise, use default formats.
        if formats.is_empty() {
            formats.extend(["pdf", "vector"].map(str::to_owned));
        }
        formats
    };

    prepare_exporters_impl(args.export.clone(), output_dir, formats)
}
