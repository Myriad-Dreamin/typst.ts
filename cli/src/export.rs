use std::path::Path;

use crate::CompileArgs;

pub static AVAILABLE_FORMATS: &[(/* format name */ &str, /* feature name */ &str)] = &[
    ("pdf", "pdf"),
    ("json", "serde-json"),
    ("rmp", "serde-rmp"),
    ("web_socket", "web-socket"),
];

pub fn prepare_exporters(
    args: CompileArgs,
    entry_file: &Path,
) -> (
    Vec<Box<dyn typst_ts_core::DocumentExporter>>,
    Vec<Box<dyn typst_ts_core::ArtifactExporter>>,
    Option<typst_ts_serde_exporter::IRArtifactExporter>,
) {
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

    let mut document_exporters: Vec<Box<dyn typst_ts_core::DocumentExporter>> = vec![];
    let mut artifact_exporters: Vec<Box<dyn typst_ts_core::ArtifactExporter>> = vec![];
    let mut ir_exporter = None;

    for f in formats {
        match f.as_str() {
            #[cfg(feature = "pdf")]
            "pdf" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("pdf");
                document_exporters.push(Box::new(typst_ts_pdf_exporter::PdfDocExporter::new_path(
                    output_path,
                )));
            }
            #[cfg(feature = "serde-json")]
            "json" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact.json");
                artifact_exporters.push(Box::new(
                    typst_ts_serde_exporter::JsonArtifactExporter::new_path(output_path),
                ));
            }
            #[cfg(feature = "serde-rmp")]
            "rmp" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact.rmp");
                artifact_exporters.push(Box::new(
                    typst_ts_serde_exporter::RmpArtifactExporter::new_path(output_path),
                ));
            }
            #[cfg(feature = "web-socket")]
            "web_socket" => {
                let mut ws_url = args.web_socket.clone();
                if ws_url.is_empty() {
                    ws_url = "127.0.0.1:23625".to_string()
                };
                artifact_exporters.push(Box::new(
                    typst_ts_ws_exporter::WebSocketArtifactExporter::new_url(ws_url),
                ));
            }
            "ir" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("artifact_ir.bin");
                ir_exporter = Some(typst_ts_serde_exporter::IRArtifactExporter::new_path(
                    output_path,
                ));
            }
            _ => {
                let found = AVAILABLE_FORMATS.iter().find(|(k, _)| **k == *f);
                if let Some((_, feat)) = found {
                    panic!("feature not enabled for format {:?}: {}", f, *feat)
                } else {
                    panic!("unknown format: {}", f)
                }
            }
        };
    }

    (document_exporters, artifact_exporters, ir_exporter)
}
