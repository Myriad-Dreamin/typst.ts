use core::fmt;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{vfs::notify::FilesystemEvent, ShadowApi};
use typst::{
    diag::{At, FileResult, Hint, SourceDiagnostic, SourceResult},
    eval::Tracer,
    foundations::Content,
    model::Document,
    syntax::Span,
    World,
};
use typst_ts_core::{
    config::compiler::EntryState, typst::prelude::*, Bytes, ImmutPath, TypstFileId,
};

pub(crate) mod diag;
#[cfg(feature = "system-compile")]
pub use diag::ConsoleDiagReporter;

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
pub mod features;
pub mod query;

pub use self::{diag::DiagnosticFormat, features::FeatureSet};

#[cfg(feature = "system-compile")]
pub type CompileDriver = CompileDriverImpl<crate::TypstSystemWorld>;

pub trait EntryManager {
    fn reset(&mut self) -> SourceResult<()> {
        Ok(())
    }

    fn workspace_root(&self) -> Option<Arc<Path>>;

    fn main_id(&self) -> Option<TypstFileId>;

    fn entry_state(&self) -> EntryState;

    fn mutate_entry(&mut self, state: EntryState) -> SourceResult<EntryState>;
}

#[derive(Clone, Default)]
pub struct CompileEnv {
    pub tracer: Option<Tracer>,
    pub features: Arc<FeatureSet>,
}

impl CompileEnv {
    pub fn configure(mut self, feature_set: FeatureSet) -> Self {
        self.features = Arc::new(feature_set);
        self
    }

    pub fn configure_shared(mut self, feature_set: Arc<FeatureSet>) -> Self {
        self.features = feature_set;
        self
    }
}

#[derive(Clone, Debug)]
pub enum CompileReport {
    Stage(TypstFileId, &'static str, crate::Time),
    CompileError(TypstFileId, EcoVec<SourceDiagnostic>, instant::Duration),
    ExportError(TypstFileId, EcoVec<SourceDiagnostic>, instant::Duration),
    CompileWarning(TypstFileId, EcoVec<SourceDiagnostic>, instant::Duration),
    CompileSuccess(TypstFileId, EcoVec<SourceDiagnostic>, instant::Duration),
}

impl CompileReport {
    pub fn compiling_id(&self) -> TypstFileId {
        match self {
            Self::Stage(id, ..)
            | Self::CompileError(id, ..)
            | Self::ExportError(id, ..)
            | Self::CompileWarning(id, ..)
            | Self::CompileSuccess(id, ..) => *id,
        }
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Stage(..) => None,
            Self::CompileError(_, _, dur)
            | Self::ExportError(_, _, dur)
            | Self::CompileWarning(_, _, dur)
            | Self::CompileSuccess(_, _, dur) => Some(*dur),
        }
    }

    pub fn diagnostics(self) -> Option<EcoVec<SourceDiagnostic>> {
        match self {
            Self::Stage(..) => None,
            Self::CompileError(_, diagnostics, ..)
            | Self::ExportError(_, diagnostics, ..)
            | Self::CompileWarning(_, diagnostics, ..)
            | Self::CompileSuccess(_, diagnostics, ..) => Some(diagnostics),
        }
    }

    /// Get the status message.
    pub fn message(&self) -> CompileReportMsg<'_> {
        CompileReportMsg(self)
    }
}

pub struct CompileReportMsg<'a>(&'a CompileReport);

impl<'a> fmt::Display for CompileReportMsg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use CompileReport::*;

        let input = self.0.compiling_id();
        match self.0 {
            Stage(_, stage, ..) => write!(f, "{:?}: {} ...", input, stage),
            CompileSuccess(_, _, duration) | CompileWarning(_, _, duration) => {
                write!(f, "{:?}: Compilation succeeded in {:?}", input, duration)
            }
            CompileError(_, _, duration) | ExportError(_, _, duration) => {
                write!(f, "{:?}: Compilation failed after {:?}", input, duration)
            }
        }
    }
}

pub trait EnvWorld {
    fn prepare_env(&mut self, _env: &mut CompileEnv) -> SourceResult<()> {
        Ok(())
    }

    fn ensure_env(&mut self) -> SourceResult<()> {
        Ok(())
    }
}

pub trait Compiler {
    type World: World + EnvWorld;

    fn world(&self) -> &Self::World;

    fn world_mut(&mut self) -> &mut Self::World;

    fn main_id(&self) -> TypstFileId;

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()>;

    /// Compile once from scratch.
    fn pure_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.reset()?;

        self.world_mut().prepare_env(env)?;

        let main_id = self.main_id();

        self.world_mut()
            .source(main_id)
            .hint(AtFile(main_id))
            .at(Span::detached())?;

        let res = match env.tracer.as_mut() {
            Some(tracer) => typst::compile(self.world(), tracer),
            None => typst::compile(self.world(), &mut Tracer::default()),
        };

        // compile document
        res.map(Arc::new)
    }

    /// With **the compilation state**, query the matches for the selector.
    fn pure_query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self::query::retrieve(self.world(), &selector, document).at(Span::detached())
    }

    /// Compile once from scratch.
    fn compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.pure_compile(env)
    }

    /// With **the compilation state**, query the matches for the selector.
    fn query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.pure_query(selector, document)
    }

    /// Iterate over the dependencies of found by the compiler.
    /// Note: reset the compiler will clear the dependencies cache.
    fn iter_dependencies<'a>(&'a self, _f: &mut dyn FnMut(&'a ImmutPath, crate::Time)) {}

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
    fn wrap_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.inner_mut().compile(env)
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
    fn pure_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.inner_mut().pure_compile(env)
    }

    #[inline]
    fn pure_query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.inner_mut().pure_query(selector, document)
    }

    #[inline]
    fn compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.wrap_compile(env)
    }

    #[inline]
    fn query(&mut self, selector: String, document: &Document) -> SourceResult<Vec<Content>> {
        self.wrap_query(selector, document)
    }

    #[inline]
    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, crate::Time)) {
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

struct AtFile(TypstFileId);

impl From<AtFile> for EcoString {
    fn from(at: AtFile) -> Self {
        eco_format!("at file {:?}", at.0)
    }
}
