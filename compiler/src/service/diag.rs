use std::io::{self, IsTerminal};
use std::path::Path;

use codespan_reporting::files::Files;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use typst::syntax::SourceId;
use typst::{diag::SourceError, World};

/// Get stderr with color support if desirable.
fn color_stream() -> StandardStream {
    StandardStream::stderr(if std::io::stderr().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    })
}

/// Print diagnostic messages to the terminal.
pub fn print_diagnostics<'files, W: World + Files<'files, FileId = SourceId>>(
    world: &'files W,
    errors: Vec<SourceError>,
) -> Result<(), codespan_reporting::files::Error> {
    let mut w = color_stream();
    let config = term::Config {
        tab_width: 2,
        ..Default::default()
    };

    for error in errors {
        // The main diagnostic.
        let range = error.range(world);
        let diag = Diagnostic::error()
            .with_message(error.message)
            .with_labels(vec![Label::primary(error.span.source(), range)]);

        term::emit(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in error.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(vec![Label::primary(
                    point.span.source(),
                    World::source(world, point.span.source()).range(point.span),
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
pub fn status(entry_file: &Path, status: Status) -> io::Result<()> {
    let input = entry_file.display();
    match status {
        Status::Compiling => log::info!("{}: compiling ...", input),
        Status::Success(duration) => {
            log::info!("{}: Compilation succeeded in {:?}", input, duration)
        }
        Status::Error(duration) => log::info!("{}: Compilation failed after {:?}", input, duration),
    };
    Ok(())
}
