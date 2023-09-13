use std::io::{self, IsTerminal};

use codespan_reporting::files::Files;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use typst::diag::Severity;
use typst::syntax::Span;
use typst::WorldExt;
use typst::{diag::SourceDiagnostic, World};

use typst::eval::eco_format;
use typst_ts_core::TypstFileId;

use super::DiagStatus;

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

    for diagnostic in errors {
        let diag = match diagnostic.severity {
            Severity::Error => Diagnostic::error(),
            Severity::Warning => Diagnostic::warning(),
        }
        .with_message(diagnostic.message.clone())
        .with_notes(
            diagnostic
                .hints
                .iter()
                .map(|e| (eco_format!("hint: {e}")).into())
                .collect(),
        )
        .with_labels(label(world, diagnostic.span).into_iter().collect());

        term::emit(&mut w, &config, world, &diag)?;

        // Stacktrace-like helper diagnostics.
        for point in diagnostic.trace {
            let message = point.v.to_string();
            let help = Diagnostic::help()
                .with_message(message)
                .with_labels(label(world, point.span).into_iter().collect());

            term::emit(&mut w, &config, world, &help)?;
        }
    }

    Ok(())
}

/// Create a label for a span.
fn label<'files, W: World + Files<'files, FileId = TypstFileId>>(
    world: &'files W,
    span: Span,
) -> Option<Label<TypstFileId>> {
    Some(Label::primary(span.id()?, world.range(span)?))
}

/// Render the status message.
pub fn status(entry_file: TypstFileId, status: DiagStatus) -> io::Result<()> {
    let input = entry_file;
    match status {
        DiagStatus::Compiling => log::info!("{:?}: compiling ...", input),
        DiagStatus::Success(duration) => {
            log::info!("{:?}: Compilation succeeded in {:?}", input, duration)
        }
        DiagStatus::Error(duration) => {
            log::info!("{:?}: Compilation failed after {:?}", input, duration)
        }
    };
    Ok(())
}
