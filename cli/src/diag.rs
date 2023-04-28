use std::io;
use std::path::Path;

use codespan_reporting::files::Files;
use codespan_reporting::term::termcolor;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use log::info;
use typst::syntax::SourceId;
use typst::{diag::SourceError, World};

/// Print diagnostic messages to the terminal.
pub fn print_diagnostics<'files, W: World + Files<'files, FileId = SourceId>>(
    world: &'files W,
    errors: Vec<SourceError>,
) -> Result<(), codespan_reporting::files::Error> {
    let mut w = StandardStream::stderr(ColorChoice::Auto);
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
    Success,
    Error,
}

impl Status {
    fn message(&self) -> &str {
        match self {
            Self::Compiling => "compiling ...",
            Self::Success => "compiled successfully",
            Self::Error => "compiled with errors",
        }
    }

    fn color(&self) -> termcolor::ColorSpec {
        let styles = term::Styles::default();
        match self {
            Self::Error => styles.header_error,
            _ => styles.header_note,
        }
    }
}

/// Clear the terminal and render the status message.
pub fn status(entry_file: &Path, status: Status) -> io::Result<()> {
    let _esc = 27 as char;
    let input = entry_file.display();
    let time = chrono::offset::Local::now();
    let _timestamp = time.format("%H:%M:%S");
    let message = status.message();
    let _color = status.color();

    info!("{}: {}", input, message);
    Ok(())
}
