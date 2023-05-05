use std::path::PathBuf;

use typst::diag::{SourceError, SourceResult};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::{exporter_builtins::GroupDocumentExporter, DocumentExporter};

use crate::diag::print_diagnostics;

pub struct CompileAction {
    pub world: TypstSystemWorld,
    pub entry_file: PathBuf,
    pub exporter: GroupDocumentExporter,
}

impl CompileAction {
    /// Print diagnostic messages to the terminal.
    pub fn print_diagnostics(
        &self,
        errors: Vec<SourceError>,
    ) -> Result<(), codespan_reporting::files::Error> {
        print_diagnostics(&self.world, errors)
    }

    /// Check whether a file system event is relevant to the world.
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

    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&self, output: typst::doc::Document) -> SourceResult<()> {
        self.exporter.export(&self.world, &output)
    }

    /// Compile once from scratch.
    pub fn once(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset();

        // checkout the entry file
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

        // compile and export document
        typst::compile(&self.world).and_then(|output| self.export(output))
    }
}
