use std::path::Path;

pub fn prepare_exporters(
    output: String,
    mut formats: Vec<String>,
    entry_file: &Path,
) -> (
    Vec<Box<dyn typst_ts_core::DocExporter>>,
    Vec<Box<dyn typst_ts_core::ArtifactExporter>>,
) {
    let output_dir = if !output.is_empty() {
        Path::new(&output)
    } else {
        entry_file.parent().unwrap()
    };
    let mut output_dir = output_dir.to_path_buf();
    output_dir.push("output");

    let mut doc_exporters: Vec<Box<dyn typst_ts_core::DocExporter>> = vec![];
    let mut artifact_exporters: Vec<Box<dyn typst_ts_core::ArtifactExporter>> = vec![];

    if formats.is_empty() {
        formats.push("pdf".to_string());
        formats.push("json".to_string());
    }
    formats.sort();
    formats.dedup();

    // if formats.contains(&"pdf".to_string()) {}

    for f in formats {
        match f.as_str() {
            #[cfg(feature = "pdf")]
            "pdf" => {
                let output_path = output_dir
                    .with_file_name(entry_file.file_name().unwrap())
                    .with_extension("pdf");
                doc_exporters.push(Box::new(typst_ts_pdf_exporter::PdfDocExporter::new_path(
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
            _ => panic!("unknown format: {}", f),
        };
    }

    (doc_exporters, artifact_exporters)
}
