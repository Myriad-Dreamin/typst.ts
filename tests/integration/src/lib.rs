pub mod wasm;

use std::path::Path;

use typst::{doc::Document, util::PathExt};
use typst_ts_compiler::{service::CompileDriver, TypstSystemWorld};
use typst_ts_core::{
    config::CompileOpts,
    exporter_builtins::{FromExporter, FsPathExporter, GroupExporter},
    AsWritable,
};
use typst_ts_pdf_exporter::PdfDocExporter;
use typst_ts_serde_exporter::JsonArtifactExporter;
use typst_ts_tir_exporter::IRArtifactExporter;

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) -> CompileDriver {
    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: workspace_dir.to_owned(),
        no_system_fonts: true,
        ..CompileOpts::default()
    });

    CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
        exporter,
    }
}

macro_rules! artifact_exporters {
    ($($exporters:expr),*) => {
        {
            let artifact_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst_ts_core::Artifact>>> = vec![
                $(Box::new($exporters)),*
            ];
            FromExporter::new(artifact_exporters)
        }
    };
}

macro_rules! document_exporters {
    ($($exporters:expr),*) => {
        {
            let document_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst::doc::Document>>> = vec![
                $(Box::new($exporters)),*
            ];
            GroupExporter::new(document_exporters)
        }
    };
}

macro_rules! ir_exporters {
    ($($exporters:expr),*) => {
        {
            let ir_exporters: Vec<Box<dyn typst_ts_core::Exporter<typst_ts_core::artifact_ir::Artifact>>> = vec![
                $(Box::new($exporters)),*
            ];
            FromExporter::new(ir_exporters)
        }
    };
}

fn artifact_json_to_path<P: AsRef<Path>>(
    path: P,
) -> FsPathExporter<AsWritable, JsonArtifactExporter> {
    FsPathExporter::new(path.as_ref().to_owned(), JsonArtifactExporter::default())
}

fn artifact_ir_to_path<P: AsRef<Path>>(path: P) -> FsPathExporter<AsWritable, IRArtifactExporter> {
    FsPathExporter::new(path.as_ref().to_owned(), IRArtifactExporter::default())
}

fn doc_pdf_to_path<P: AsRef<Path>>(path: P) -> FsPathExporter<Vec<u8>, PdfDocExporter> {
    FsPathExporter::new(path.as_ref().to_owned(), PdfDocExporter::default())
}

pub struct ArtifactBundle {
    pub driver: CompileDriver,
    pub json: std::path::PathBuf,
    pub tir: std::path::PathBuf,
    pub pdf: std::path::PathBuf,
}

pub struct ArtifactCompiler {
    pub corpus_root: std::path::PathBuf,
    pub artifact_dir: std::path::PathBuf,
}

impl ArtifactCompiler {
    pub fn compile(&self, workspace_dir: String, entry_file: String) -> ArtifactBundle {
        let entry_file_base = Path::new(&entry_file);

        let real_entry_file_path = self.corpus_root.join(entry_file_base);
        let real_workspace_dir = self.corpus_root.join(workspace_dir);

        let artifact_dir = &self.artifact_dir;

        let json_artifact_file_path =
            artifact_dir.join(entry_file_base.with_extension("artifact.json"));
        let tir_file_path = artifact_dir.join(entry_file_base.with_extension("artifact.tir.bin"));
        let pdf_file_path = artifact_dir.join(entry_file_base.with_extension("pdf"));

        let artifact_dir_to_create = json_artifact_file_path.parent().unwrap().to_owned();
        std::fs::create_dir_all(artifact_dir_to_create).unwrap();

        let mut driver = get_driver(
            &real_workspace_dir,
            &real_entry_file_path,
            document_exporters![
                artifact_exporters![artifact_json_to_path(json_artifact_file_path.clone())],
                ir_exporters![artifact_ir_to_path(tir_file_path.clone())],
                doc_pdf_to_path(pdf_file_path.clone())
            ],
        );

        driver.once().unwrap();

        ArtifactBundle {
            driver,
            json: json_artifact_file_path.normalize(),
            tir: tir_file_path.normalize(),
            pdf: pdf_file_path.normalize(),
        }
    }
}
