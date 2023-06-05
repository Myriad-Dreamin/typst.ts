use std::path::{Path, PathBuf};

use typst_ts_core::{
    exporter_builtins::{FromExporter, FsPathExporter, GroupExporter},
    program_meta::REPORT_BUG_MESSAGE,
    Artifact, AsWritable,
};

use crate::CompileArgs;

type GroupDocExporter = GroupExporter<typst::doc::Document>;

/// builtin formats should be enabled by default, and non-builtin formats should be
pub static AVAILABLE_FORMATS: &[(/* format name */ &str, /* feature hint */ &str)] = &[
    ("ast", REPORT_BUG_MESSAGE),
    ("ir", REPORT_BUG_MESSAGE),
    ("nothing", REPORT_BUG_MESSAGE),
    ("pdf", "pdf"),
    ("json", "serde-json"),
    ("rmp", "serde-rmp"),
    ("web_socket", "web-socket"),
];

fn panic_not_available_formats(f: String) -> ! {
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

fn prepare_exporters_impl(
    output_dir: PathBuf,
    formats: Vec<String>,
    ws_url: String,
    entry_file: &Path,
) -> GroupDocExporter {
    type DocExporter = Box<dyn typst_ts_core::Exporter<typst::doc::Document>>;
    type ArtExporter = Box<dyn typst_ts_core::Exporter<typst_ts_core::Artifact>>;
    type IRExporter = Box<dyn typst_ts_core::Exporter<typst_ts_core::artifact_ir::Artifact>>;

    let mut document_exporters: Vec<DocExporter> = vec![];
    let mut artifact_exporters: Vec<ArtExporter> = vec![];
    let mut ir_exporters: Vec<IRExporter> = vec![];

    for f in formats {
        match f.as_str() {
            "ast" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("ast.ansi.text");
                document_exporters.push(Box::new(FsPathExporter::new(
                    output_path,
                    typst_ts_ast_exporter::AstExporter::default(),
                )));
            }
            "ir" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact.tir.bin");

                let exp = typst_ts_tir_exporter::IRArtifactExporter::default();
                let exp = FsPathExporter::new(output_path, exp);
                ir_exporters.push(Box::new(exp));
            }
            #[cfg(feature = "pdf")]
            "pdf" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("pdf");
                document_exporters.push(Box::new(FsPathExporter::new(
                    output_path,
                    typst_ts_pdf_exporter::PdfDocExporter::default(),
                )));
            }
            #[cfg(feature = "serde-json")]
            "json" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact.json");
                artifact_exporters.push(Box::new(FsPathExporter::<AsWritable, _>::new(
                    output_path,
                    typst_ts_serde_exporter::JsonExporter::<Artifact>::default(),
                )));
            }
            #[cfg(feature = "serde-rmp")]
            "rmp" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact.rmp");
                artifact_exporters.push(Box::new(FsPathExporter::new(
                    output_path,
                    typst_ts_serde_exporter::RmpArtifactExporter::default(),
                )));
            }
            #[cfg(feature = "web-socket")]
            "web_socket" => {
                let mut ws_url = ws_url.clone();
                if ws_url.is_empty() {
                    ws_url = "127.0.0.1:23625".to_string()
                };
                artifact_exporters.push(Box::new(
                    typst_ts_ws_exporter::WebSocketArtifactExporter::new_url(ws_url),
                ));
            }
            "nothing" => (),
            _ => panic_not_available_formats(f),
        };
    }

    if !artifact_exporters.is_empty() {
        document_exporters.push(Box::new(FromExporter::new(artifact_exporters)));
    }

    if !ir_exporters.is_empty() {
        document_exporters.push(Box::new(FromExporter::new(ir_exporters)));
    }

    GroupExporter::new(document_exporters)
}

pub fn prepare_exporters(args: &CompileArgs, entry_file: &Path) -> GroupDocExporter {
    let output_dir = {
        let output = args.output.clone();
        let output_dir = if !output.is_empty() {
            Path::new(&output)
        } else {
            entry_file.parent().unwrap()
        };
        let mut output_dir = output_dir.to_path_buf();
        output_dir.push("output");

        output_dir
    };

    let formats = {
        let mut formats = args.format.clone();
        if !args.web_socket.is_empty() {
            formats.push("web_socket".to_string());
        }
        if formats.is_empty() {
            formats.push("pdf".to_string());
            formats.push("json".to_string());
        }
        formats.sort();
        formats.dedup();
        formats
    };

    prepare_exporters_impl(output_dir, formats, args.web_socket.clone(), entry_file)
}
