use std::{path::PathBuf, sync::Arc};

use crate::TypstSystemWorld;
use typst::diag::{SourceError, SourceResult};
use typst_ts_core::{exporter_builtins::GroupExporter, exporter_utils::map_err, Exporter};
use typst_ts_svg_exporter::{serialize_multi_doc_standalone, DynamicLayoutSvgExporter};

use super::diag;

/// CompileDriver is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriver {
    /// World that has access to the file system.
    pub world: TypstSystemWorld,
    /// Path to the entry file.
    pub entry_file: PathBuf,
    /// Exporter to use, which will consume the output of the compiler.
    pub exporter: GroupExporter<typst::doc::Document>,
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
        self.exporter.export(&self.world, Arc::new(output))
    }

    /// Compile once from scratch.
    pub fn once(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset();

        // checkout the entry file
        let entry_file = self.entry_file.clone();
        self.world.main = self
            .world
            .resolve(&entry_file)
            .map_err(|e| map_err(&self.world, e))?;

        // compile and export document
        typst::compile(&self.world).and_then(|output| self.export(output))
    }

    /// Compile once from scratch.
    pub fn once_dynamic(&mut self) -> SourceResult<()> {
        // checkout the entry file
        let entry_file = self.entry_file.clone();
        // todo: hexo svg
        let content = { std::fs::read_to_string(&entry_file).expect("Could not read file") };
        // #let ts_page_width = 595.28pt

        use typst::geom::Abs;

        let mut svg_exporter = DynamicLayoutSvgExporter::default();
        let base_layout = Abs::pt(592.28);

        // for each 10pt we rerender once
        let instant_begin = std::time::Instant::now();
        for i in 0..40 {
            let instant = std::time::Instant::now();
            // replace layout
            let current_width = base_layout - Abs::pt(i as f64 * 10.0);

            let to_layout: String = format!("#let ts_page_width = {:2}pt", current_width.to_pt());
            println!(
                "rerendering {} at {:?}, {to_layout}",
                i,
                instant - instant_begin
            );

            // reset the world caches
            self.world.reset();

            let dyn_content = content
                .clone()
                .replace("#let ts_page_width = 595.28pt", &to_layout);

            match self.world.resolve_with(&entry_file, &dyn_content) {
                Ok(id) => {
                    self.world.main = id;
                }
                Err(e) => return Err(map_err(&self.world, e)),
            }

            // compile and export document
            let output = Arc::new(typst::compile(&self.world).unwrap());
            svg_exporter.render(current_width, output);
            println!(
                "rerendered {} at {:?}, {}",
                i,
                instant - instant_begin,
                svg_exporter.debug_stat()
            );
        }

        let module_output = entry_file.with_extension("multi.sir.bin");

        let (doc, glyphs) = svg_exporter.finalize();

        std::fs::write(module_output, serialize_multi_doc_standalone(doc, glyphs)).unwrap();

        let instant = std::time::Instant::now();
        println!("rerendering finished at {:?}", instant - instant_begin);
        Ok(())
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
