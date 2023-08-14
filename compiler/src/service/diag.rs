use std::io::{self, IsTerminal};

use codespan_reporting::files::Files;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use typst::{diag::SourceDiagnostic, World};

use typst_ts_core::TypstFileId;

/// Get stderr with color support if desirable.
fn color_stream() -> StandardStream {
    StandardStream::stderr(if std::io::stderr().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    })
}

/// Print diagnostic messages to the terminal.
pub fn print_diagnostics<'files, W: World + Files<'files, FileId = TypstFileId>>(
    world: &'files W,
    errors: Vec<SourceDiagnostic>,
) -> Result<(), codespan_reporting::files::Error> {
    let mut w = color_stream();
    let config = term::Config {
        tab_width: 2,
        ..Default::default()
    };

    for error in errors {
        // The main diagnostic.
        let source = typst::World::source(world, error.span.id()).ok();
        let range = source
            .map(|source| source.range(error.span))
            .unwrap_or(0..0);
        let diag = Diagnostic::error()
            .with_message(error.message)
            .with_labels(vec![Label::primary(error.span.id(), range)]);

        term::emit(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in error.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(vec![Label::primary(
                    point.span.id(),
                    world.range(point.span),
                )]);

            term::emit(&mut w, &config, world, &help)?;
        }
    }

    Ok(())
}

/// The status in which the watcher can be.
pub enum Status {
    Compiling,
    Success(std::time::Duration),
    Error(std::time::Duration),
}

/// Render the status message.
pub fn status(entry_file: TypstFileId, status: Status) -> io::Result<()> {
    let input = entry_file;
    match status {
        Status::Compiling => log::info!("{}: compiling ...", input),
        Status::Success(duration) => {
            log::info!("{}: Compilation succeeded in {:?}", input, duration)
        }
        Status::Error(duration) => log::info!("{}: Compilation failed after {:?}", input, duration),
    };
    Ok(())
}
