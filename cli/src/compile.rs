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

    /// Check whether a file system event is relevant to the world.
    pub fn relevant(&mut self, event: &notify::Event) -> bool {
        use notify::event::ModifyKind;
        use notify::EventKind;

        macro_rules! fs_event_must_relevant {
            () => {
                // create a file in workspace
                EventKind::Create(_) |
                // rename a file in workspace
                EventKind::Modify(ModifyKind::Name(_))
            };
        }
        macro_rules! fs_event_may_relevant {
            () => {
                // remove/modify file in workspace
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Data(_)) |
                // unknown manipulation in workspace
                EventKind::Any | EventKind::Modify(ModifyKind::Any)
            };
        }
        macro_rules! fs_event_never_relevant {
            () => {
                // read/write meta event
                EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) |
                // `::notify` internal events other event that we cannot identify
                EventKind::Other | EventKind::Modify(ModifyKind::Other)
            };
        }

        match &event.kind {
            fs_event_must_relevant!() => true,
            fs_event_may_relevant!() => event.paths.iter().any(|path| self.world.dependant(path)),
            fs_event_never_relevant!() => false,
        }
    }
}
