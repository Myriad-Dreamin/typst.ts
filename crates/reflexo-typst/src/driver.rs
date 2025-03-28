use typst::diag::{SourceDiagnostic, SourceResult, Warned};

use crate::{
    diag::print_diagnostics,
    world::{CompilerFeat, CompilerWorld},
    CompileReport, DiagnosticFormat,
};

#[derive(Default, Clone)]
pub struct DiagnosticHandler {
    /// The diagnostic format to use.
    pub diagnostic_format: DiagnosticFormat,
    /// Whether to print the compile status.
    pub print_compile_status: bool,
}

impl DiagnosticHandler {
    pub fn status(&self, rep: &CompileReport) {
        if self.print_compile_status {
            log::info!("{}", rep.message());
        }
    }

    pub fn report_compiled<T, F: CompilerFeat>(
        &self,
        world: &CompilerWorld<F>,
        res: Warned<SourceResult<T>>,
    ) -> Option<T> {
        let (result, diag) = match res.output {
            Ok(doc) => (Some(doc), res.warnings),
            Err(diag) => (None, diag),
        };
        if !diag.is_empty() {
            self.report(world, diag.iter());
        }

        result
    }

    pub fn report<'d, F: CompilerFeat>(
        &self,
        world: &CompilerWorld<F>,
        diagnostics: impl Iterator<Item = &'d SourceDiagnostic>,
    ) {
        let _err = print_diagnostics(world, diagnostics, self.diagnostic_format);
        // todo: log in browser compiler
        #[cfg(feature = "system-compile")]
        if _err.is_err() {
            log::error!("failed to print diagnostics: {_err:?}");
        }
    }
}

// todo: Print that a package downloading is happening.
// fn print_downloading(_spec: &PackageSpec) -> std::io::Result<()> {
// let mut w = color_stream();
// let styles = term::Styles::default();

// w.set_color(&styles.header_help)?;
// write!(w, "downloading")?;

// w.reset()?;
// writeln!(w, " {spec}")
// Ok(())
// }
