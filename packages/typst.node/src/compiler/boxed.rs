#![deny(clippy::all)]

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use typst_ts_compiler::{
    CompileDriver, CompileEnv, Compiler, EntryManager, EntryReader, PureCompiler, ShadowApi,
    TaskInputs, TypstSystemWorld,
};
use typst_ts_core::{
    config::compiler::{EntryState, MEMORY_MAIN_ENTRY},
    error::{prelude::*, TypstSourceDiagnostic},
    error_once,
    foundations::Content,
    typst::prelude::*,
    Bytes, TypstDocument, TypstFileId,
};

use crate::{error::NodeTypstCompileResult, map_node_error, CompileDocArgs, NodeError};

// <World = TypstSystemWorld>
pub trait NodeCompilerTrait: Compiler
where
    Self::W: EntryManager + ShadowApi,
{
}

// <World = TypstSystemWorld>
pub struct BoxedCompiler(Box<CompileDriver<PureCompiler<TypstSystemWorld>>>);

// <World = TypstSystemWorld>
impl Deref for BoxedCompiler {
    type Target = CompileDriver<PureCompiler<TypstSystemWorld>>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for BoxedCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

// <World = TypstSystemWorld>
impl From<CompileDriver<PureCompiler<TypstSystemWorld>>> for BoxedCompiler {
    fn from(value: CompileDriver<PureCompiler<TypstSystemWorld>>) -> Self {
        Self(Box::new(value))
    }
}

type SourceResult<T> = Result<T, EcoVec<TypstSourceDiagnostic>>;

impl BoxedCompiler {
    /// Create a temporary world for compiling once.
    /// Should be pure and not affect the current world.
    pub fn setup_compiler_by(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> napi::Result<TypstSystemWorld, NodeError> {
        let universe = self.universe();
        let new_state = {
            if let Some(main_file_content) = compile_by.main_file_content {
                if compile_by.main_file_path.is_some() {
                    return Err(map_node_error(error_once!(
                        "main file content and path cannot be specified at the same time"
                    )));
                }

                let new_entry = universe
                    .entry_state()
                    .select_in_workspace(*MEMORY_MAIN_ENTRY);

                let content = Bytes::from(main_file_content.as_bytes());
                // TODO: eliminate the side effect of shadow mapping safely
                if let Err(err) = self
                    .universe_mut()
                    .map_shadow_by_id(*MEMORY_MAIN_ENTRY, content)
                {
                    return Err(map_node_error(error_once!("cannot map shadow", err: err)));
                }

                Some(new_entry)
            } else if let Some(main_file_path) = compile_by.main_file_path {
                if compile_by.main_file_content.is_some() {
                    return Err(map_node_error(error_once!(
                        "main file content and path cannot be specified at the same time"
                    )));
                }

                let abs_fp = std::path::absolute(main_file_path.as_str());
                let fp = abs_fp
                    .as_ref()
                    .map(|p| std::path::Path::new(p))
                    .map_err(|e| {
                        map_node_error(error_once!("cannot absolutize the main file path", err: e))
                    })?;
                universe
                    .entry_state()
                    .try_select_path_in_workspace(fp, true)
                    .map_err(map_node_error)?
            } else {
                None
            }
        };

        Ok(self.universe.snapshot_with(Some(TaskInputs {
            entry: new_state,
            ..Default::default()
        })))
    }

    pub fn compile_raw(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> napi::Result<NodeTypstCompileResult, NodeError> {
        let world = self.setup_compiler_by(compile_by)?;

        if self.0.universe().entry_state().is_inactive() {
            Err(map_node_error(error_once!("entry file is not set")))
        } else {
            // FIXME: This is implementation detail, use a better way from
            // the compiler driver.
            let c = &mut self.0.compiler;
            c.ensure_main(&world).map_err(map_node_error)?;
            Ok(c.compile(&world, &mut CompileEnv::default()).into())
        }
    }
}

/// A blanket implementation for all `CompileMiddleware`.
/// If you want to wrap a compiler, you should override methods in
/// `CompileMiddleware`.
impl Compiler for BoxedCompiler {
    type W = TypstSystemWorld;

    #[inline]
    fn reset(&mut self) -> SourceResult<()> {
        self.0.reset()
    }

    #[inline]
    fn pure_compile(
        &mut self,
        world: &TypstSystemWorld,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<TypstDocument>> {
        self.0.compiler.pure_compile(world, env)
    }

    #[inline]
    fn pure_query(
        &mut self,
        world: &TypstSystemWorld,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        self.0.compiler.pure_query(world, selector, document)
    }

    #[inline]
    fn compile(
        &mut self,
        world: &TypstSystemWorld,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<TypstDocument>> {
        self.0.compiler.compile(world, env)
    }

    #[inline]
    fn query(
        &mut self,
        world: &TypstSystemWorld,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        self.0.compiler.query(world, selector, document)
    }
}
