use std::path::PathBuf;

use typst::diag::{SourceError, SourceResult};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::{
    exporter_builtins::GroupDocumentExporter, exporter_utils::map_err, DocumentExporter,
};

use crate::diag;

/// CompileDriver is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriver {
    /// World that has access to the file system.
    pub world: TypstSystemWorld,
    /// Path to the entry file.
    pub entry_file: PathBuf,
    /// Exporter to use, which will consume the output of the compiler.
    pub exporter: GroupDocumentExporter,
}

impl CompileDriver {
    /// Print diagnostic messages to the terminal.
    fn print_diagnostics(
        &self,
        errors: Vec<SourceError>,
    ) -> Result<(), codespan_reporting::files::Error> {
        diag::print_diagnostics(&self.world, errors)
    }

    /// Print status message to the terminal.
    fn print_status<const WITH_STATUS: bool>(&self, status: diag::Status) {
        if !WITH_STATUS {
            return;
        }
        diag::status(&self.entry_file, status).unwrap();
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
            Err(e) => return Err(map_err(&self.world, e)),
        }

        // compile and export document
        typst::compile(&self.world).and_then(|output| self.export(output))
    }

    /// Compile once from scratch and print (optional) status and diagnostics to the terminal.
    pub fn once_diag<const WITH_STATUS: bool>(&mut self) -> bool {
        self.print_status::<WITH_STATUS>(diag::Status::Compiling);
        match self.once() {
            Ok(_) => {
                self.print_status::<WITH_STATUS>(diag::Status::Success);
                true
            }
            Err(errs) => {
                self.print_status::<WITH_STATUS>(diag::Status::Error);
                self.print_diagnostics(*errs).unwrap();
                false
            }
        }
    }

    /// Check whether a file system event is relevant to the world.
    pub fn relevant(&self, event: &notify::Event) -> bool {
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
