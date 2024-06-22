use core::fmt;
use std::sync::Arc;

use reflexo_world::EntryReader;
use typst::{
    diag::{At, Hint, SourceDiagnostic, SourceResult},
    eval::Tracer,
    foundations::Content,
    model::Document,
    syntax::Span,
    World,
};
use typst_ts_core::{typst::prelude::*, TypstFileId};

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
pub type CompileDriver<C> = CompileDriverImpl<C, reflexo_world::system::SystemCompilerFeat>;

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
    type W: World;

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()>;

    fn ensure_main(&self, world: &Self::W) -> SourceResult<()>
    where
        Self::W: EntryReader,
    {
        let main_id = world
            .main_id()
            .ok_or_else(|| eco_format!("no entry file"))
            .at(Span::detached())?;

        world
            .source(main_id)
            .hint(AtFile(main_id))
            .at(Span::detached())?;

        Ok(())
    }

    /// Compile once from scratch.
    fn pure_compile(
        &mut self,
        world: &Self::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<Document>> {
        self.reset()?;

        let res = match env.tracer.as_mut() {
            Some(tracer) => typst::compile(world, tracer),
            None => typst::compile(world, &mut Tracer::default()),
        };

        // compile document
        res.map(Arc::new)
    }

    /// With **the compilation state**, query the matches for the selector.
    fn pure_query(
        &mut self,
        world: &Self::W,
        selector: String,
        document: &Document,
    ) -> SourceResult<Vec<Content>> {
        self::query::retrieve(world, &selector, document).at(Span::detached())
    }

    /// Compile once from scratch.
    fn compile(&mut self, world: &Self::W, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.pure_compile(world, env)
    }

    /// With **the compilation state**, query the matches for the selector.
    fn query(
        &mut self,
        world: &Self::W,
        selector: String,
        document: &Document,
    ) -> SourceResult<Vec<Content>> {
        self.pure_query(world, selector, document)
    }
}

pub type PureCompiler<W> = std::marker::PhantomData<fn(W)>;

impl<W: World> Compiler for PureCompiler<W> {
    type W = W;

    fn reset(&mut self) -> SourceResult<()> {
        Ok(())
    }
}

pub trait CompileMiddleware {
    type Compiler: Compiler;

    fn inner(&self) -> &Self::Compiler;

    fn inner_mut(&mut self) -> &mut Self::Compiler;

    /// Hooked reset the compilation state
    fn wrap_reset(&mut self) -> SourceResult<()> {
        self.inner_mut().reset()
    }

    /// Hooked compile once from scratch.
    fn wrap_compile(
        &mut self,
        world: &<<Self as CompileMiddleware>::Compiler as Compiler>::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<Document>> {
        self.inner_mut().compile(world, env)
    }

    /// With **the compilation state**, hooked query the matches for the
    /// selector.
    fn wrap_query(
        &mut self,
        world: &<<Self as CompileMiddleware>::Compiler as Compiler>::W,
        selector: String,
        document: &Document,
    ) -> SourceResult<Vec<Content>> {
        self.inner_mut().query(world, selector, document)
    }
}

/// A blanket implementation for all `CompileMiddleware`.
/// If you want to wrap a compiler, you should override methods in
/// `CompileMiddleware`.
impl<T: CompileMiddleware> Compiler for T {
    type W = <<T as CompileMiddleware>::Compiler as Compiler>::W;

    #[inline]
    fn reset(&mut self) -> SourceResult<()> {
        self.wrap_reset()
    }

    #[inline]
    fn pure_compile(
        &mut self,
        world: &Self::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<Document>> {
        self.inner_mut().pure_compile(world, env)
    }

    #[inline]
    fn pure_query(
        &mut self,
        world: &Self::W,
        selector: String,
        document: &Document,
    ) -> SourceResult<Vec<Content>> {
        self.inner_mut().pure_query(world, selector, document)
    }

    #[inline]
    fn compile(&mut self, world: &Self::W, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        self.wrap_compile(world, env)
    }

    #[inline]
    fn query(
        &mut self,
        world: &Self::W,
        selector: String,
        document: &Document,
    ) -> SourceResult<Vec<Content>> {
        self.wrap_query(world, selector, document)
    }
}

struct AtFile(TypstFileId);

impl From<AtFile> for EcoString {
    fn from(at: AtFile) -> Self {
        eco_format!("at file {:?}", at.0)
    }
}
