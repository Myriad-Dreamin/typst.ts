use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{vfs::notify::FilesystemEvent, ShadowApi};
use typst::{
    diag::{At, FileResult, Hint, SourceDiagnostic, SourceResult},
    doc::Document,
    eval::Tracer,
    model::Content,
    syntax::Span,
    World,
};
use typst_library::prelude::{eco_format, EcoString};
use typst_ts_core::{Bytes, ImmutPath, TypstFileId};

// todo: remove cfg feature here
#[cfg(feature = "system-compile")]
pub(crate) mod diag;

#[cfg(feature = "system-watch")]
pub(crate) mod watch;
#[cfg(feature = "system-watch")]
pub use watch::*;

pub(crate) mod driver;
pub use driver::*;

#[cfg(feature = "system-watch")]
pub(crate) mod compile;
#[cfg(feature = "system-watch")]
pub use compile::*;

pub(crate) mod export;
pub use export::*;
pub mod query;

#[cfg(feature = "system-compile")]
pub(crate) mod session;
#[cfg(feature = "system-compile")]
pub use session::*;

#[cfg(feature = "system-compile")]
pub type CompileDriver = CompileDriverImpl<crate::TypstSystemWorld>;

pub trait WorkspaceProvider {
    fn reset(&mut self) -> SourceResult<()> {
        Ok(())
    }

    fn workspace_root(&self) -> Arc<Path>;

    fn set_main_id(&mut self, id: TypstFileId);
}

pub trait Compiler {
    type World: World;

    fn world(&self) -> &Self::World;

    fn world_mut(&mut self) -> &mut Self::World;

    fn main_id(&self) -> TypstFileId;

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()>;

    /// Compile once from scratch.
    fn pure_compile(&mut self) -> SourceResult<Arc<Document>> {
        self.reset()?;

        let main_id = self.main_id();

        self.world_mut()
            .source(main_id)
            .hint(AtFile(main_id))
            .at(Span::detached())?;

        let mut tracer = Tracer::default();
        // compile and export document
        typst::compile(self.world(), &mut tracer).map(Arc::new)
    }

    /// With **the compilation state**, query the matches for the selector.
    fn pure_query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self::query::retrieve(self.world(), &selector, document).at(Span::detached())
    }

    /// Compile once from scratch.
    fn compile(&mut self) -> SourceResult<Arc<Document>> {
        self.pure_compile()
    }

    /// With **the compilation state**, query the matches for the selector.
    fn query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.pure_query(selector, document)
    }

    /// Iterate over the dependencies of found by the compiler.
    /// Note: reset the compiler will clear the dependencies cache.
    fn iter_dependencies<'a>(&'a self, _f: &mut dyn FnMut(&'a ImmutPath, instant::SystemTime)) {}

    fn notify_fs_event(&mut self, _event: FilesystemEvent) {}

    /// Determine whether the event is relevant to the compiler.
    /// The default implementation is conservative, which means that
    /// `MaybeRelevant` implies `MustRelevant`.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    fn relevant(&self, event: &notify::Event) -> bool {
        self._relevant(event).unwrap_or(true)
    }

    /// The default implementation of `relevant` method, which performs a
    /// simple check on the event kind.
    /// It returns following values:
    /// - `Some(true)`: the event must be relevant to the compiler.
    /// - `Some(false)`: the event must not be relevant to the compiler.
    /// - `None`: the event may be relevant to the compiler.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    fn _relevant(&self, event: &notify::Event) -> Option<bool> {
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

        return match &event.kind {
            fs_event_must_relevant!() => Some(true),
            fs_event_may_relevant!() => None,
            fs_event_never_relevant!() => Some(false),
        };

        // assert that all cases are covered
        const _: () = match EventKind::Any {
            fs_event_must_relevant!() | fs_event_may_relevant!() | fs_event_never_relevant!() => {}
        };
    }
}

pub trait CompileMiddleware {
    type Compiler: Compiler;

    fn inner(&self) -> &Self::Compiler;

    fn inner_mut(&mut self) -> &mut Self::Compiler;

    fn wrap_main_id(&self) -> TypstFileId {
        self.inner().main_id()
    }

    /// Hooked world access
    fn wrap_world(&self) -> &<Self::Compiler as Compiler>::World {
        self.inner().world()
    }

    /// Hooked world mut access
    fn wrap_world_mut(&mut self) -> &mut <Self::Compiler as Compiler>::World {
        self.inner_mut().world_mut()
    }

    /// Hooked reset the compilation state
    fn wrap_reset(&mut self) -> SourceResult<()> {
        self.inner_mut().reset()
    }

    /// Hooked compile once from scratch.
    fn wrap_compile(&mut self) -> SourceResult<Arc<Document>> {
        self.inner_mut().compile()
    }

    /// With **the compilation state**, hooked query the matches for the
    /// selector.
    fn wrap_query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.inner_mut().query(selector, document)
    }
}

/// A blanket implementation for all `CompileMiddleware`.
/// If you want to wrap a compiler, you should override methods in
/// `CompileMiddleware`.
impl<T: CompileMiddleware> Compiler for T {
    type World = <T::Compiler as Compiler>::World;

    #[inline]
    fn world(&self) -> &Self::World {
        self.wrap_world()
    }

    #[inline]
    fn world_mut(&mut self) -> &mut Self::World {
        self.wrap_world_mut()
    }

    #[inline]
    fn main_id(&self) -> TypstFileId {
        self.wrap_main_id()
    }

    #[inline]
    fn reset(&mut self) -> SourceResult<()> {
        self.wrap_reset()
    }

    #[inline]
    fn pure_compile(&mut self) -> SourceResult<Arc<Document>> {
        self.inner_mut().pure_compile()
    }

    #[inline]
    fn pure_query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.inner_mut().pure_query(selector, document)
    }

    #[inline]
    fn compile(&mut self) -> SourceResult<Arc<Document>> {
        self.wrap_compile()
    }

    #[inline]
    fn query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.wrap_query(selector, document)
    }

    #[inline]
    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, instant::SystemTime)) {
        self.inner().iter_dependencies(f)
    }

    #[inline]
    fn notify_fs_event(&mut self, event: crate::vfs::notify::FilesystemEvent) {
        self.inner_mut().notify_fs_event(event)
    }
}

impl<T: CompileMiddleware> ShadowApi for T
where
    T::Compiler: ShadowApi,
{
    #[inline]
    fn _shadow_map_id(&self, _file_id: TypstFileId) -> FileResult<PathBuf> {
        self.inner()._shadow_map_id(_file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.inner().shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.inner_mut().reset_shadow()
    }

    #[inline]
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.inner().map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&self, path: &Path) -> FileResult<()> {
        self.inner().unmap_shadow(path)
    }
}

/// The status in which the watcher can be.
pub enum DiagStatus {
    Stage(&'static str),
    Success(std::time::Duration),
    Error(std::time::Duration),
}

pub trait DiagObserver {
    /// Print diagnostic messages to the terminal.
    fn print_diagnostics(
        &self,
        errors: ecow::EcoVec<SourceDiagnostic>,
    ) -> Result<(), codespan_reporting::files::Error>;

    /// Print status message to the terminal.
    fn print_status<const WITH_STATUS: bool>(&self, status: DiagStatus);

    /// Run inner function with print (optional) status and diagnostics to the
    /// terminal (for compilation).
    #[deprecated(note = "use `with_stage_diag` instead")]
    fn with_compile_diag<const WITH_STATUS: bool, T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> Option<T>;

    /// Run inner function with print (optional) status and diagnostics to the
    /// terminal.
    fn with_stage_diag<const WITH_STATUS: bool, T>(
        &mut self,
        stage: &'static str,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> Option<T>;
}

#[cfg(feature = "system-compile")]
impl<C: Compiler> DiagObserver for C
where
    C::World: for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    /// Print diagnostic messages to the terminal.
    fn print_diagnostics(
        &self,
        errors: ecow::EcoVec<SourceDiagnostic>,
    ) -> Result<(), codespan_reporting::files::Error> {
        diag::print_diagnostics(self.world(), errors)
    }

    /// Print status message to the terminal.
    fn print_status<const WITH_STATUS: bool>(&self, status: DiagStatus) {
        if !WITH_STATUS {
            return;
        }
        diag::status(self.main_id(), status).unwrap();
    }

    /// Run inner function with print (optional) status and diagnostics to the
    /// terminal (for compilation).
    fn with_compile_diag<const WITH_STATUS: bool, T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> Option<T> {
        self.with_stage_diag::<WITH_STATUS, _>("compiling", f)
    }

    /// Run inner function with print (optional) status and diagnostics to the
    /// terminal (for stages).
    fn with_stage_diag<const WITH_STATUS: bool, T>(
        &mut self,
        stage: &'static str,
        f: impl FnOnce(&mut Self) -> SourceResult<T>,
    ) -> Option<T> {
        self.print_status::<WITH_STATUS>(DiagStatus::Stage(stage));
        let start = instant::Instant::now();
        match f(self) {
            Ok(val) => {
                self.print_status::<WITH_STATUS>(DiagStatus::Success(start.elapsed()));
                Some(val)
            }
            Err(errs) => {
                self.print_status::<WITH_STATUS>(DiagStatus::Error(start.elapsed()));
                let _err = self.print_diagnostics(errs);
                // todo: log in browser compiler
                #[cfg(feature = "system-compile")]
                if _err.is_err() {
                    log::error!("failed to print diagnostics: {:?}", _err);
                }
                None
            }
        }
    }
}

struct AtFile(TypstFileId);

impl From<AtFile> for EcoString {
    fn from(at: AtFile) -> Self {
        eco_format!("at file {:?}", at.0)
    }
}
