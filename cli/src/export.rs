use std::path::{Path, PathBuf};

use typst_ts_core::{
    exporter_builtins::{FromExporter, FsPathExporter, GroupExporter},
    program_meta::REPORT_BUG_MESSAGE,
    AsWritable,
};
use typst_ts_svg_exporter::DefaultExportFeature;

use crate::CompileArgs;

type GroupDocExporter = GroupExporter<typst::doc::Document>;

/// builtin formats should be enabled by default, and non-builtin formats should be
pub static AVAILABLE_FORMATS: &[(/* format name */ &str, /* feature hint */ &str)] = &[
    ("ast", REPORT_BUG_MESSAGE),
    ("ir", REPORT_BUG_MESSAGE),
    ("json", "serde-json"),
    ("json_glyphs", "serde-json"),
    ("nothing", REPORT_BUG_MESSAGE),
    ("pdf", "pdf"),
    ("svg", "svg"),
    ("sir", "svg"),
    ("rmp", "serde-rmp"),
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
fn prepare_exporters_impl(out: PathBuf, mut formats: Vec<String>) -> GroupDocExporter {
    let mut artifact: ExporterVec<Artifact> = vec![];
    let mut doc: ExporterVec<Doc> = vec![];
    let mut glyph_pack: ExporterVec<GlyphPack> = vec![];
    let mut ir: ExporterVec<IR> = vec![];

    /// connect export flow from $x to $y
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
    }

    // sink exporters according to the given formats
    {
        formats.sort();
        formats.dedup();
        #[rustfmt::skip]
        formats.iter().map(String::as_str).for_each(|f| match f {
            "nothing"     => (),
            "ast"         => sink_path!(WithAst as _ as doc, out @@ "ast.ansi.text"),
            "ir"          => sink_path!(WithIR as _ as ir, out @@ "artifact.tir.bin"),
            #[cfg(feature = "pdf")]
            "pdf"         => sink_path!(WithPdf as _ as doc, out @@ "pdf"),
            #[cfg(feature = "serde-json")]
            "json"        => sink_path!(WithJson<_> as AsWritable as artifact, out @@ "artifact.json"),
            #[cfg(feature = "serde-json")]
            "json_glyphs" => sink_path!(WithJson<_> as AsWritable as glyph_pack, out @@ "glyphs.json"),
            #[cfg(feature = "serde-rmp")]
            "rmp"         => sink_path!(WithRmp as _ as artifact, out @@ "artifact.rmp"),
            #[cfg(feature = "svg")]
            "svg"         => sink_path!(WithSvg as _ as doc, out @@ "artifact.svg.html"),
            #[cfg(feature = "svg")]
            "sir"         => sink_path!(WithSIR as _ as doc, out @@ "artifact.sir.bin"),
            _             => exit_by_unknown_format(f),
        });
    }
    {
        sink_flow!(doc -> artifact);
        sink_flow!(doc -> ir);
        sink_flow!(doc -> glyph_pack);
    }
    return GroupExporter::new(doc);

    type Artifact = typst_ts_core::artifact::Artifact;
    type Doc = typst::doc::Document;
    type GlyphPack = typst_ts_core::font::FontGlyphPackBundle;
    type IR = typst_ts_core::artifact_ir::Artifact;

    type WithAst = typst_ts_ast_exporter::AstExporter;
    type WithIR = typst_ts_tir_exporter::IRArtifactExporter;
    type WithJson<T> = typst_ts_serde_exporter::JsonExporter<T>;
    type WithPdf = typst_ts_pdf_exporter::PdfDocExporter;
    type WithRmp = typst_ts_serde_exporter::RmpArtifactExporter;
    type WithSvg = typst_ts_svg_exporter::SvgExporter<DefaultExportFeature>;
    type WithSIR = typst_ts_svg_exporter::SvgModuleExporter;

    type ExporterVec<T> = Vec<Box<dyn typst_ts_core::Exporter<T> + Send>>;
}

/// Prepare exporters from command line arguments.
pub fn prepare_exporters(args: &CompileArgs, entry_file: &Path) -> GroupDocExporter {
    let output_dir = {
        // If output is specified, use it.
        let dir = (!args.output.is_empty()).then(|| Path::new(&args.output));
        // Otherwise, use the parent directory of the entry file.
        let dir = dir.unwrap_or_else(|| entry_file.parent().expect("entry_file has no parent"));
        dir.join(entry_file.file_name().expect("entry_file has no file name"))
    };

    let formats = {
        // If formats are specified, use them.
        let mut formats = args.format.clone();
        // Otherwise, use default formats.
        if formats.is_empty() {
            formats.extend(["pdf", "json"].map(str::to_owned));
        }
        formats
    };

    prepare_exporters_impl(output_dir, formats)
}
