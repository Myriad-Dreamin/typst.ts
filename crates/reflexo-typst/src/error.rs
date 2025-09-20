pub use reflexo::error::*;
pub use typst::diag::FileError as TypstFileError;
pub use typst::diag::SourceDiagnostic as TypstSourceDiagnostic;

use core::fmt;

use ecow::eco_format;
use reflexo::debug_loc::{LspPosition, LspRange};
use reflexo::path::unix_slash;
use typst::syntax::{FileId, Source, Span};

use crate::vfs::{WorkspaceResolution, WorkspaceResolver};

#[derive(Clone, Debug)]
pub enum CompileReport {
    Suspend,
    Stage(FileId, &'static str, crate::Time),
    CompileError(FileId, usize, reflexo::time::Duration),
    ExportError(FileId, usize, reflexo::time::Duration),
    CompileSuccess(FileId, usize, reflexo::time::Duration),
}

impl CompileReport {
    pub fn compiling_id(&self) -> Option<FileId> {
        Some(match self {
            Self::Suspend => return None,
            Self::Stage(id, ..)
            | Self::CompileError(id, ..)
            | Self::ExportError(id, ..)
            | Self::CompileSuccess(id, ..) => *id,
        })
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Suspend | Self::Stage(..) => None,
            Self::CompileError(_, _, dur)
            | Self::ExportError(_, _, dur)
            | Self::CompileSuccess(_, _, dur) => Some(*dur),
        }
    }

    pub fn diagnostics_size(self) -> Option<usize> {
        match self {
            Self::Suspend | Self::Stage(..) => None,
            Self::CompileError(_, diagnostics, ..)
            | Self::ExportError(_, diagnostics, ..)
            | Self::CompileSuccess(_, diagnostics, ..) => Some(diagnostics),
        }
    }

    /// Get the status message.
    pub fn message(&self) -> CompileReportMsg<'_> {
        CompileReportMsg(self)
    }
}

pub struct CompileReportMsg<'a>(&'a CompileReport);

impl fmt::Display for CompileReportMsg<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CompileReport::*;

        let input = WorkspaceResolver::display(self.0.compiling_id());
        match self.0 {
            Suspend => write!(f, "suspended"),
            Stage(_, stage, ..) => write!(f, "{input:?}: {stage} ..."),
            CompileSuccess(_, warnings, duration) => {
                if *warnings == 0 {
                    write!(f, "{input:?}: compilation succeeded in {duration:?}")
                } else {
                    write!(
                        f,
                        "{input:?}: compilation succeeded with {warnings} warnings in {duration:?}",
                    )
                }
            }
            CompileError(_, _, duration) | ExportError(_, _, duration) => {
                write!(f, "{input:?}: compilation failed after {duration:?}")
            }
        }
    }
}

struct DiagMsgFmt<'a>(&'a TypstSourceDiagnostic);

impl fmt::Display for DiagMsgFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.message)?;
        if !self.0.hints.is_empty() {
            f.write_str(", hints: ")?;
            f.write_str(&self.0.hints.join(", "))?;
        }

        Ok(())
    }
}

struct PosFmt<'a>(&'a typst::diag::Tracepoint);

impl fmt::Display for PosFmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            typst::diag::Tracepoint::Call(Some(name)) => write!(f, "while calling {name}"),
            typst::diag::Tracepoint::Call(None) => write!(f, "while calling closure"),
            typst::diag::Tracepoint::Show(name) => write!(f, "while showing {name}"),
            typst::diag::Tracepoint::Import => write!(f, "import"),
        }
    }
}

fn resolve_source_span(
    s: Span,
    world: Option<&dyn typst::World>,
) -> (String, String, Option<LspRange>) {
    let mut package = String::new();
    let mut path = String::new();
    let mut range = None;

    if let Some(id) = s.id() {
        match WorkspaceResolver::resolve(id) {
            Ok(WorkspaceResolution::Package) => {
                package = id.package().unwrap().to_string();
                path = unix_slash(id.vpath().as_rooted_path());
            }
            Ok(WorkspaceResolution::Rootless | WorkspaceResolution::UntitledRooted(..)) => {
                path = unix_slash(id.vpath().as_rooted_path());
            }
            Ok(WorkspaceResolution::Workspace(workspace)) => {
                path = id
                    .vpath()
                    .resolve(&workspace.path())
                    .as_deref()
                    .map(unix_slash)
                    .unwrap_or_default();
            }
            Err(..) => {}
        }

        if let Some((rng, src)) = world
            .and_then(|world| world.source(id).ok())
            .and_then(|src| Some((src.find(s)?.range(), src)))
        {
            let resolve_off =
                |src: &Source, off: usize| src.byte_to_line(off).zip(src.byte_to_column(off));
            range = Some(LspRange {
                start: resolve_off(&src, rng.start)
                    .map(|(l, c)| LspPosition::new(l as u32, c as u32))
                    .unwrap_or_default(),
                end: resolve_off(&src, rng.end)
                    .map(|(l, c)| LspPosition::new(l as u32, c as u32))
                    .unwrap_or_default(),
            });
        }
    }

    (package, path, range)
}

pub fn diag_from_std(
    diag: &TypstSourceDiagnostic,
    world: Option<&dyn typst::World>,
) -> DiagMessage {
    // arguments.push(("code", diag.code.to_string()));

    let (package, path, range) = resolve_source_span(diag.span, world);

    DiagMessage {
        package,
        path,
        message: eco_format!("{}", DiagMsgFmt(diag)),
        severity: match diag.severity {
            typst::diag::Severity::Error => DiagSeverity::Error,
            typst::diag::Severity::Warning => DiagSeverity::Warning,
        },
        range,
    }
}

/// Convert typst.ts diagnostic message with trace messages
pub fn long_diag_from_std<'a>(
    diag: &'a TypstSourceDiagnostic,
    world: Option<&'a dyn typst::World>,
) -> impl Iterator<Item = DiagMessage> + 'a {
    let base = Some(diag_from_std(diag, world));

    base.into_iter().chain(diag.trace.iter().map(move |trace| {
        let (package, path, range) = resolve_source_span(trace.span, world);
        DiagMessage {
            package,
            path,
            message: eco_format!("{}", PosFmt(&trace.v)),
            severity: DiagSeverity::Hint,
            range,
        }
    }))
}

pub trait ErrorConverter {
    // todo: file_id to path
    /// Convert typst.ts diagnostic to error
    /// It has a simple implementation.
    /// If you want to customize it, you can implement it yourself.
    fn convert_typst(&self, world: &dyn typst::World, diag: TypstSourceDiagnostic) -> Error {
        let mut arguments = Vec::new();

        let msg = diag_from_std(&diag, Some(world));
        arguments.push(("severity", msg.severity.to_string()));
        arguments.push(("package", msg.package));
        arguments.push(("path", msg.path));
        if let Some(range) = msg.range {
            arguments.push(("start_line", range.start.line.to_string()));
            arguments.push(("start_column", range.start.character.to_string()));
            arguments.push(("end_line", range.end.line.to_string()));
            arguments.push(("end_column", range.end.character.to_string()));
        }

        Error::new(
            "typst",
            ErrKind::Msg(msg.message),
            Some(arguments.into_boxed_slice()),
        )
    }
}
