//! reflexo-typst library.
//!
//! This library is used to compile Typst code into a document and export it
//! into various artifacts.
//! See <https://github.com/Myriad-Dreamin/typst.ts/tree/main/exporter> for
//! more information about the available exporters.
//!
//! The library consists of three parts:
//! - `model`: low-level abstraction specific to the compiler, which defines:
//!   - [`font::FontSlot`]: the way to load a font.
//!   - [`vfs::AccessModel`]: how the compiler accesses a storage.
//!   - [`package::PackageRegistry`]: how the compiler obtains data about a
//!     package.
//!
//! - [`world`]: The world is the core part of the library, which maintains all
//!   the data for typst compilation.
//!   - [`vfs::Vfs`]: retrieving [`vfs::AccessModel`], provides a virtual file
//!     system for the [`world::CompilerWorld`]
//!   - [`world::CompilerWorld`]: retrieving [`world::CompilerFeat`], provides a
//!     common implementation of [`::typst::World`].
//!
//! - [`compile`]: Convenient services over [`world::CompilerWorld`], which also
//!   shows how to use the [`world::CompilerWorld`].
//!   - [`CompileDriver`]: A driver for the compiler. Examples:
//!     - Single thread (Sync): <https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/main.rs>
//!     - Multiple thread (Async): <https://github.com/Enter-tainer/typst-preview-vscode/blob/main/src/main.rs>

// Core type system/concepts of typst-ts.
// #![warn(missing_docs)]
// #![warn(missing_debug_implementations)]
// #![warn(missing_copy_implementations)]

mod concepts;
pub use concepts::*;

// Core data structures of typst-ts.
pub mod config;
pub mod error;

// Core mechanism of typst-ts.
pub(crate) mod exporter;

#[cfg(feature = "ast")]
pub use exporter::ast::{dump_ast, AstExporter};

pub use exporter::json::JsonExporter;

use ::typst::engine::Sink;
#[cfg(feature = "pdf")]
pub use exporter::pdf::PdfDocExporter;
#[cfg(feature = "pdf")]
pub use typst_pdf::pdf;

#[cfg(feature = "svg")]
pub use exporter::svg::*;
#[cfg(feature = "svg")]
pub use reflexo_vec2svg as svg;

pub use exporter::text::TextExporter;

pub use reflexo_typst2vec as vector;
pub use reflexo_typst2vec::debug_loc;
pub use reflexo_typst2vec::hash;

pub use exporter::{builtins as exporter_builtins, utils as exporter_utils};
pub use exporter::{
    DynExporter, DynGenericExporter, DynPolymorphicExporter, Exporter, GenericExporter,
    GenericTransformer, Transformer,
};
// pub use font::{FontLoader, FontResolver, FontSlot};
pub use reflexo::*;

pub mod build_info {
    /// The version of the reflexo-typst crate.
    pub static VERSION: &str = env!("CARGO_PKG_VERSION");
}

pub mod program_meta {
    /// inform the user that this is a bug.
    pub const REPORT_BUG_MESSAGE: &str =
        "This is a bug, please report to https://github.com/Myriad-Dreamin/typst.ts/issues/new";
}

pub mod diag;
mod driver;
pub mod eval;
mod export;
pub mod features;
pub mod query;
mod utils;

/// font things about compiler.
pub use world::font;

/// time things about compiler.
pub use reflexo::time;
/// A vfs implementation for compiler.
pub use reflexo_vfs as vfs;
/// A common implementation of [`::typst::World`]
pub use reflexo_world as world;
pub use time::Time;
/// package things about compiler.
pub use world::package;
/// Diff and parse the source code.
pub use world::parser;
pub use world::*;

#[cfg(feature = "system-watch")]
mod watch;
#[cfg(feature = "system-watch")]
pub use compile::*;
#[cfg(feature = "system-watch")]
pub use watch::*;

#[cfg(feature = "system-watch")]
mod compile;
#[cfg(feature = "system-watch")]
pub mod task;
#[cfg(feature = "system-compile")]
pub use diag::ConsoleDiagReporter;
#[cfg(feature = "system-compile")]
pub type CompileDriver<C> = CompileDriverImpl<C, reflexo_world::system::SystemCompilerFeat>;

pub use self::{diag::DiagnosticFormat, features::FeatureSet};
pub use driver::*;
pub use export::*;

use core::fmt;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::typst::prelude::*;
use ::typst::{
    diag::{At, Hint, SourceDiagnostic, SourceResult},
    foundations::Content,
    model::Document,
    syntax::Span,
    utils::Deferred,
    World,
};

#[derive(Clone, Default)]
pub struct CompileEnv {
    pub sink: Option<Sink>,
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
    Suspend,
    Stage(TypstFileId, &'static str, crate::Time),
    CompileError(
        TypstFileId,
        EcoVec<SourceDiagnostic>,
        reflexo::time::Duration,
    ),
    ExportError(
        TypstFileId,
        EcoVec<SourceDiagnostic>,
        reflexo::time::Duration,
    ),
    CompileWarning(
        TypstFileId,
        EcoVec<SourceDiagnostic>,
        reflexo::time::Duration,
    ),
    CompileSuccess(
        TypstFileId,
        EcoVec<SourceDiagnostic>,
        reflexo::time::Duration,
    ),
}

impl CompileReport {
    pub fn compiling_id(&self) -> Option<TypstFileId> {
        Some(match self {
            Self::Suspend => return None,
            Self::Stage(id, ..)
            | Self::CompileError(id, ..)
            | Self::ExportError(id, ..)
            | Self::CompileWarning(id, ..)
            | Self::CompileSuccess(id, ..) => *id,
        })
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Suspend | Self::Stage(..) => None,
            Self::CompileError(_, _, dur)
            | Self::ExportError(_, _, dur)
            | Self::CompileWarning(_, _, dur)
            | Self::CompileSuccess(_, _, dur) => Some(*dur),
        }
    }

    pub fn diagnostics(self) -> Option<EcoVec<SourceDiagnostic>> {
        match self {
            Self::Suspend | Self::Stage(..) => None,
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
            Suspend => write!(f, "suspended"),
            Stage(_, stage, ..) => write!(f, "{:?}: {} ...", input, stage),
            CompileSuccess(_, _, duration) | CompileWarning(_, _, duration) => {
                write!(f, "{:?}: compilation succeeded in {:?}", input, duration)
            }
            CompileError(_, _, duration) | ExportError(_, _, duration) => {
                write!(f, "{:?}: compilation failed after {:?}", input, duration)
            }
        }
    }
}

type CompileRawResult = Deferred<(SourceResult<Arc<TypstDocument>>, CompileEnv)>;
type DocState = std::sync::OnceLock<CompileRawResult>;

/// A signal that possibly triggers an export.
///
/// Whether to export depends on the current state of the document and the user
/// settings.
#[derive(Debug, Clone, Copy)]
pub struct ExportSignal {
    /// Whether the revision is annotated by memory events.
    pub by_mem_events: bool,
    /// Whether the revision is annotated by file system events.
    pub by_fs_events: bool,
    /// Whether the revision is annotated by entry update.
    pub by_entry_update: bool,
}

pub struct CompileSnapshot<F: CompilerFeat> {
    /// The export signal for the document.
    pub flags: ExportSignal,
    /// Using env
    pub env: CompileEnv,
    /// Using world
    pub world: Arc<CompilerWorld<F>>,
    /// Compiling the document.
    doc_state: Arc<DocState>,
    /// The last successfully compiled document.
    pub success_doc: Option<Arc<TypstDocument>>,
}

impl<F: CompilerFeat + 'static> CompileSnapshot<F> {
    fn start(&self) -> &CompileRawResult {
        self.doc_state.get_or_init(|| {
            let w = self.world.clone();
            let mut env = self.env.clone();
            Deferred::new(move || {
                let w = w.as_ref();
                let mut c = std::marker::PhantomData;
                let res = c.ensure_main(w).and_then(|_| c.compile(w, &mut env));
                (res, env)
            })
        })
    }

    pub fn task(mut self, inputs: TaskInputs) -> Self {
        'check_changed: {
            if let Some(entry) = &inputs.entry {
                if *entry != self.world.entry_state() {
                    break 'check_changed;
                }
            }
            if let Some(inputs) = &inputs.inputs {
                if inputs.clone() != self.world.inputs() {
                    break 'check_changed;
                }
            }

            return self;
        };

        self.world = Arc::new(self.world.task(inputs));
        self.doc_state = Arc::new(OnceLock::new());

        self
    }

    pub fn compile(&self) -> CompiledArtifact<F> {
        let (doc, env) = self.start().wait().clone();
        CompiledArtifact {
            signal: self.flags,
            world: self.world.clone(),
            env,
            doc,
            success_doc: self.success_doc.clone(),
        }
    }
}

impl<F: CompilerFeat> Clone for CompileSnapshot<F> {
    fn clone(&self) -> Self {
        Self {
            flags: self.flags,
            env: self.env.clone(),
            world: self.world.clone(),
            doc_state: self.doc_state.clone(),
            success_doc: self.success_doc.clone(),
        }
    }
}

pub struct CompiledArtifact<F: CompilerFeat> {
    /// All the export signal for the document.
    pub signal: ExportSignal,
    /// Used world
    pub world: Arc<CompilerWorld<F>>,
    /// Used env
    pub env: CompileEnv,
    pub doc: SourceResult<Arc<TypstDocument>>,
    success_doc: Option<Arc<TypstDocument>>,
}

impl<F: CompilerFeat> Clone for CompiledArtifact<F> {
    fn clone(&self) -> Self {
        Self {
            signal: self.signal,
            world: self.world.clone(),
            env: self.env.clone(),
            doc: self.doc.clone(),
            success_doc: self.success_doc.clone(),
        }
    }
}

impl<F: CompilerFeat> CompiledArtifact<F> {
    pub fn success_doc(&self) -> Option<Arc<TypstDocument>> {
        self.doc
            .as_ref()
            .ok()
            .cloned()
            .or_else(|| self.success_doc.clone())
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

        let res = match env.sink.as_mut() {
            Some(sink) => ::typst::compile(world),
            None => ::typst::compile(world),
        };

        // compile document
        res.output.map(Arc::new)
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

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_hash128() {
        assert_eq!(typst::utils::hash128(&0u32), reflexo::hash::hash128(&0u32));
    }
}
