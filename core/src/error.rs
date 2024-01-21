use core::fmt;

use reflexo::debug_loc::CharRange;
pub use reflexo::error::*;

use reflexo::path::unix_slash;
use typst::syntax::Source;

pub use typst::diag::SourceDiagnostic as TypstSourceDiagnostic;

pub use typst::diag::FileError as TypstFileError;

struct DiagMsgFmt<'a>(&'a TypstSourceDiagnostic);

impl<'a> fmt::Display for DiagMsgFmt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.message)?;
        if !self.0.hints.is_empty() {
            f.write_str(", hints: ")?;
            f.write_str(&self.0.hints.join(", "))?;
        }
        if !self.0.trace.is_empty() {
            write!(f, "{:?}", self.0.trace)?;
        }

        Ok(())
    }
}

pub fn diag_from_std(diag: TypstSourceDiagnostic, world: Option<&dyn typst::World>) -> DiagMessage {
    // arguments.push(("code", diag.code.to_string()));

    let mut package = String::new();
    let mut path = String::new();
    let mut range = None;

    if let Some(id) = diag.span.id() {
        if let Some(pkg) = id.package() {
            package = pkg.to_string();
        };
        path = unix_slash(id.vpath().as_rooted_path());

        if let Some((rng, src)) = world
            .and_then(|world| world.source(id).ok())
            .and_then(|src| Some((src.find(diag.span)?.range(), src)))
        {
            let resolve_off =
                |src: &Source, off: usize| src.byte_to_line(off).zip(src.byte_to_column(off));
            range = Some(CharRange {
                start: resolve_off(&src, rng.start).into(),
                end: resolve_off(&src, rng.end).into(),
            });
        }
    }

    DiagMessage {
        package,
        path,
        message: format!("{}", DiagMsgFmt(&diag)),
        severity: match diag.severity {
            typst::diag::Severity::Error => DiagSeverity::Error,
            typst::diag::Severity::Warning => DiagSeverity::Warning,
        },
        range,
    }
}

pub trait ErrorConverter {
    // todo: file_id to path
    /// Convert typst.ts diagnostic to error
    /// It has a simple implementation.
    /// If you want to customize it, you can implement it yourself.
    fn convert_typst(&self, world: &dyn typst::World, diag: TypstSourceDiagnostic) -> Error {
        let mut arguments = Vec::new();

        let msg = diag_from_std(diag, Some(world));
        arguments.push(("severity", msg.severity.to_string()));
        arguments.push(("package", msg.package));
        arguments.push(("path", msg.path));
        if let Some(range) = msg.range {
            arguments.push(("start_line", range.start.line.to_string()));
            arguments.push(("start_column", range.start.column.to_string()));
            arguments.push(("end_line", range.end.line.to_string()));
            arguments.push(("end_column", range.end.column.to_string()));
        }

        Error::new(
            "typst",
            ErrKind::Msg(msg.message),
            arguments.into_boxed_slice(),
        )
    }
}
