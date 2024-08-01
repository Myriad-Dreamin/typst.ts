use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{Datelike, Timelike};
use typst::{
    diag::{At, FileError},
    syntax::Span,
};
use typst_ts_ast_exporter::dump_ast;
use typst_ts_core::{
    exporter_builtins::{FsPathExporter, GroupExporter},
    program_meta::REPORT_BUG_MESSAGE,
    Transformer, TypstDatetime,
};
use typst_ts_svg_exporter::DefaultExportFeature;

use crate::{utils::current_dir, CompileArgs, ExportArgs};

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
                WithPdf::default().with_ctime(args.creation_timestamp.and_then(convert_datetime))
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

    type WithAst = AstExporter;
    // type WithJson<T> = typst_ts_serde_exporter::JsonExporter<T>;
    type WithPdf = typst_ts_pdf_exporter::PdfDocExporter;
    // type WithRmp<T> = typst_ts_serde_exporter::RmpExporter<T>;
    type WithSvg = typst_ts_svg_exporter::PureSvgExporter;
    type WithSvgHtml = typst_ts_svg_exporter::SvgExporter<DefaultExportFeature>;
    type WithSIR = typst_ts_svg_exporter::SvgModuleExporter;
    type WithText = typst_ts_text_exporter::TextExporter;

    type ExporterVec<T> = Vec<Box<dyn typst_ts_core::Exporter<T> + Send + Sync>>;
}

/// Prepare exporters from command line arguments.
pub fn prepare_exporters(args: &CompileArgs, entry_file: Option<&Path>) -> GroupDocExporter {
    let output_dir = {
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

/// Convert [`chrono::DateTime`] to [`TypstDatetime`]
fn convert_datetime(date_time: chrono::DateTime<chrono::Utc>) -> Option<TypstDatetime> {
    TypstDatetime::from_ymd_hms(
        date_time.year(),
        date_time.month().try_into().ok()?,
        date_time.day().try_into().ok()?,
        date_time.hour().try_into().ok()?,
        date_time.minute().try_into().ok()?,
        date_time.second().try_into().ok()?,
    )
}

#[derive(Debug, Clone, Default)]
pub struct AstExporter {}

impl<W> Transformer<(Arc<typst::model::Document>, W)> for AstExporter
where
    W: std::io::Write,
{
    fn export(
        &self,
        world: &dyn typst::World,
        (_output, writer): (Arc<typst::model::Document>, W),
    ) -> typst::diag::SourceResult<()> {
        let mut writer = std::io::BufWriter::new(writer);

        let src = world.main();
        let path = src.id().vpath().as_rootless_path();
        dump_ast(&path.display().to_string(), &src, &mut writer)
            .map_err(|e| FileError::from_io(e, path))
            .at(Span::detached())?;

        writer.flush().unwrap();

        Ok(())
    }
}
