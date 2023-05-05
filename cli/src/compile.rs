use std::path::PathBuf;

use typst::diag::SourceError;
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::exporter_utils::collect_err;

use crate::diag::print_diagnostics;

pub struct CompileAction {
    pub world: TypstSystemWorld,
    pub entry_file: PathBuf,
    pub document_exporters: Vec<Box<dyn typst_ts_core::DocumentExporter>>,
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

                for f in &self.document_exporters {
                    collect_err(&mut errors, f.export(&self.world, &document))
                }

                errors
            }
            Err(errors) => *errors,
        }
    }
}
