pub mod wasm;

use std::path::Path;

use reflexo_typst::config::{entry::EntryOpts, CompileOpts};
use reflexo_typst::path::PathClean;
use reflexo_typst::task::ExportPdfTask;
use reflexo_typst::{ExportWebSvgModuleTask, TypstSystemUniverse};
use typst_ts_cli::export::ReflexoTaskBuilder;

fn get_driver(workspace_dir: &Path, entry_file_path: &Path) -> TypstSystemUniverse {
    let verse = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    verse.with_entry_file(entry_file_path.to_owned())
}

pub struct ArtifactBundle {
    pub verse: TypstSystemUniverse,
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

        let exporter = {
            let mut tb = ReflexoTaskBuilder::new();
            tb.add_pdf(ExportPdfTask::default());
            tb.add_web_svg_module(ExportWebSvgModuleTask::default());
            tb.set_output_path(artifact_dir.join(entry_file_base));

            tb.build()
        };

        let verse = get_driver(&real_workspace_dir, &real_entry_file_path);
        (exporter)(&verse.computation()).unwrap();

        ArtifactBundle {
            verse,
            // todo: duplicated path construction
            tir: sir_file_path.clean(),
            pdf: pdf_file_path.clean(),
        }
    }
}
