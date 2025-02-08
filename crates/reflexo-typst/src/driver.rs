use typst::diag::{SourceDiagnostic, SourceResult, Warned};

use crate::DiagnosticFormat;
use crate::{
    diag::print_diagnostics,
    world::{CompilerFeat, CompilerWorld},
    CompileReport,
};

#[derive(Default, Clone)]
pub struct DiagnosticHandler {
    /// The diagnostic format to use.
    pub diagnostic_format: DiagnosticFormat,
    /// Whether to print the compile status.
    pub print_compile_status: bool,
}

impl DiagnosticHandler {
    // todo: check status
    // impl<W: World, C: Compiler<W = W>> CompileMiddleware for CompileReporter<C,
    // W> where
    //     W: EntryReader,
    // {
    //     type Compiler = C;

    //     fn inner(&self) -> &Self::Compiler {
    //         &self.compiler
    //     }

    //     fn inner_mut(&mut self) -> &mut Self::Compiler {
    //         &mut self.compiler
    //     }

    //     fn wrap_compile(
    //         &mut self,
    //         world: &C::W,
    //         env: &mut CompileEnv,
    //     ) -> SourceResult<Warned<Arc<Document>>> {
    //         let start = reflexo::time::now();
    //         // todo unwrap main id
    //         let id = world.main_id().unwrap();
    //         if WITH_COMPILING_STATUS_FEATURE.retrieve(&env.features) {
    //             let rep = CompileReport::Stage(id, "compiling", start);
    //             let rep = Arc::new((env.features.clone(), rep));
    //             // we currently ignore export error here
    //             let _ = self.reporter.export(world, rep);
    //         }

    //         let doc = self.inner_mut().compile(world, env);

    //         let elapsed = start.elapsed().unwrap_or_default();

    //         let rep;

    //         let doc = match doc {
    //             Ok(doc) => {
    //                 rep = CompileReport::CompileSuccess(id, doc.warnings.clone(),
    // elapsed);

    //                 Ok(doc)
    //             }
    //             Err(err) => {
    //                 rep = CompileReport::CompileError(id, err, elapsed);
    //                 Err(eco_vec![])
    //             }
    //         };

    //         let rep = Arc::new((env.features.clone(), rep));
    //         // we currently ignore export error here
    //         let _ = self.reporter.export(world, rep);

    //         doc
    //     }
    // }

    pub fn status(&self, rep: CompileReport) {
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
