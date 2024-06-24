pub mod wasm;

use std::path::Path;

use typst_ts_compiler::{
    CompileDriver, CompileExporter, PureCompiler, TypstSystemUniverse, TypstSystemWorld,
};
use typst_ts_core::{
    config::{compiler::EntryOpts, CompileOpts},
    exporter_builtins::{FsPathExporter, GroupExporter},
    path::PathClean,
    TypstDocument,
};
use typst_ts_pdf_exporter::PdfDocExporter;
use typst_ts_svg_exporter::SvgModuleExporter;

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<TypstDocument>,
) -> CompileDriver<CompileExporter<PureCompiler<TypstSystemWorld>>> {
    let world = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    let world = world.with_entry_file(entry_file_path.to_owned());
    CompileDriver::new(CompileExporter::default().with_exporter(exporter), world)
}

macro_rules! document_exporters {
    ($($exporters:expr),*) => {
        {
            let document_exporters: Vec<Box<dyn typst_ts_core::Exporter<TypstDocument> + Send + Sync>> = vec![
                $(Box::new($exporters)),*
            ];
            GroupExporter::new(document_exporters)
        }
    };
}

fn artifact_ir_to_path<P: AsRef<Path>>(path: P) -> FsPathExporter<Vec<u8>, SvgModuleExporter> {
    FsPathExporter::new(path.as_ref().to_owned(), SvgModuleExporter::default())
}

fn doc_pdf_to_path<P: AsRef<Path>>(path: P) -> FsPathExporter<Vec<u8>, PdfDocExporter> {
    FsPathExporter::new(path.as_ref().to_owned(), PdfDocExporter::default())
}

pub struct ArtifactBundle {
    pub driver: CompileDriver<CompileExporter<PureCompiler<TypstSystemWorld>>>,
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

        let sir_file_path = artifact_dir.join(entry_file_base.with_extension("artifact.sir.in"));
        let pdf_file_path = artifact_dir.join(entry_file_base.with_extension("pdf"));

        let artifact_dir_to_create = sir_file_path.parent().unwrap().to_owned();
        std::fs::create_dir_all(artifact_dir_to_create).unwrap();

        let mut driver = get_driver(
            &real_workspace_dir,
            &real_entry_file_path,
            document_exporters![
                artifact_ir_to_path(sir_file_path.clone()),
                doc_pdf_to_path(pdf_file_path.clone())
            ],
        );

        driver.compile(&mut Default::default()).unwrap();

        ArtifactBundle {
            driver,
            tir: sir_file_path.clean(),
            pdf: pdf_file_path.clean(),
        }
    }
}
