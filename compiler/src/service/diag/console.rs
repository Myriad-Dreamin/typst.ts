use std::io::{self, IsTerminal};
use std::sync::Arc;

use codespan_reporting::files::Files;
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use typst::diag::{Severity, SourceResult};
use typst::syntax::Span;
use typst::WorldExt;
use typst::{diag::SourceDiagnostic, World};

use typst::eval::eco_format;
use typst_ts_core::{typst::prelude::*, GenericExporter, PhantomParamData, TakeAs, TypstFileId};

use crate::service::features::{
    CompileFeature, FeatureSet, DIAG_FMT_FEATURE, WITH_COMPILING_STATUS_FEATURE,
};
use crate::service::CompileReport;

use super::{DiagStatus, DiagnosticFormat};

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
    errors: EcoVec<SourceDiagnostic>,
    diagnostic_format: DiagnosticFormat,
) -> Result<(), codespan_reporting::files::Error> {
    let mut w = match diagnostic_format {
        DiagnosticFormat::Human => color_stream(),
        DiagnosticFormat::Short => StandardStream::stderr(ColorChoice::Never),
    };

    let mut config = term::Config {
        tab_width: 2,
        ..Default::default()
    };
    if diagnostic_format == DiagnosticFormat::Short {
        config.display_style = term::DisplayStyle::Short;
    }

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
        DiagStatus::Stage(stage) => log::info!("{:?}: {} ...", input, stage),
        DiagStatus::Success(duration) => {
            log::info!("{:?}: Compilation succeeded in {:?}", input, duration)
        }
        DiagStatus::Error(duration) => {
            log::info!("{:?}: Compilation failed after {:?}", input, duration)
        }
    };
    Ok(())
}

pub struct ConsoleDiagReporter<W>(PhantomParamData<W>);

impl<W> Default for ConsoleDiagReporter<W>
where
    W: for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    fn default() -> Self {
        Self(PhantomParamData::default())
    }
}

impl<X> GenericExporter<CompileReport> for ConsoleDiagReporter<X>
where
    X: World + for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    type W = X;

    fn export(&self, world: &Self::W, output: Arc<CompileReport>) -> SourceResult<()> {
        self.export(world, Arc::new((Default::default(), output.take())))
    }
}

impl<X> GenericExporter<(Arc<FeatureSet>, CompileReport)> for ConsoleDiagReporter<X>
where
    X: World + for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    type W = X;

    fn export(
        &self,
        world: &Self::W,
        output: Arc<(Arc<FeatureSet>, CompileReport)>,
    ) -> SourceResult<()> {
        let (features, output) = output.take();

        if WITH_COMPILING_STATUS_FEATURE.retrieve(&features) {
            use CompileReport::*;
            status(
                output.compiling_id(),
                match &output {
                    Stage(_, stage, ..) => DiagStatus::Stage(stage),
                    CompileError(..) | ExportError(..) => {
                        DiagStatus::Error(output.duration().unwrap())
                    }
                    CompileSuccess(..) | CompileWarning(..) => {
                        DiagStatus::Success(output.duration().unwrap())
                    }
                },
            )
            .unwrap();
        }

        if let Some(diag) = output.diagnostics() {
            let _err = print_diagnostics(world, diag, DIAG_FMT_FEATURE.retrieve(&features));
            // todo: log in browser compiler
            #[cfg(feature = "system-compile")]
            if _err.is_err() {
                log::error!("failed to print diagnostics: {:?}", _err);
            }
        }

        Ok(())
    }
}
