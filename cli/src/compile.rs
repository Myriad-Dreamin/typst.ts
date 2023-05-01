use std::path::PathBuf;
use std::sync::Arc;

use typst::diag::{SourceError, SourceResult};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::{artifact_ir::Artifact as IRArtifact, Artifact};

use crate::diag::print_diagnostics;

pub struct CompileAction {
    pub world: TypstSystemWorld,
    pub entry_file: PathBuf,
    pub doc_exporters: Vec<Box<dyn typst_ts_core::DocumentExporter>>,
    pub artifact_exporters: Vec<Box<dyn typst_ts_core::ArtifactExporter>>,
    pub ir_artifact_exporter: Option<typst_ts_serde_exporter::IRArtifactExporter>,
}

impl CompileAction {
    /// Print diagnostic messages to the terminal.
    pub fn print_diagnostics(
        &self,
        errors: Vec<SourceError>,
    ) -> Result<(), codespan_reporting::files::Error> {
        print_diagnostics(&self.world, errors)
    }

    /// Check if the given event is relevant to the world.
    pub fn relevant(&mut self, event: &notify::Event) -> bool {
        match &event.kind {
            notify::EventKind::Any => {}
            notify::EventKind::Access(_) => return false,
            notify::EventKind::Create(_) => return true,
            notify::EventKind::Modify(kind) => match kind {
                notify::event::ModifyKind::Any => {}
                notify::event::ModifyKind::Data(_) => {}
                notify::event::ModifyKind::Metadata(_) => return false,
                notify::event::ModifyKind::Name(_) => return true,
                notify::event::ModifyKind::Other => return false,
            },
            notify::EventKind::Remove(_) => {}
            notify::EventKind::Other => return false,
        }

        event.paths.iter().any(|path| self.world.dependant(path))
    }

    /// Compile once.
    pub fn once(&mut self) -> Vec<SourceError> {
        self.world.reset();

        let entry_file = self.entry_file.clone();
        let content = { std::fs::read_to_string(&entry_file).expect("Could not read file") };
        match self.world.resolve_with(&entry_file, &content) {
            Ok(id) => {
                self.world.main = id;
            }
            Err(e) => {
                panic!("handler unresolved main error {e}")
            }
        }

        match typst::compile(&self.world) {
            Ok(document) => {
                let mut errors = vec![];
                let mut collect_err = |res: SourceResult<()>| {
                    if let Err(errs) = res {
                        for e in *errs {
                            errors.push(e);
                        }
                    }
                };

                for f in &self.doc_exporters {
                    collect_err(f.export(&self.world, &document))
                }

                if !self.artifact_exporters.is_empty() {
                    let artifact = Arc::new(Artifact::from(document.clone()));
                    for f in &self.artifact_exporters {
                        collect_err(f.export(&self.world, artifact.clone()))
                    }
                }

                if let Some(ir_artifact_exporter) = &self.ir_artifact_exporter {
                    let artifact = Arc::new(IRArtifact::from(document));
                    collect_err(ir_artifact_exporter.export(&self.world, artifact.clone()));
                }

                errors
            }
            Err(errors) => *errors,
        }
    }
}
