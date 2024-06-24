#![deny(clippy::all)]

use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};
use typst_ts_compiler::{
    CompileDriver, CompileEnv, Compiler, EntryManager, EntryReader, PureCompiler, ShadowApi,
    TypstSystemWorld,
};
use typst_ts_core::{
    config::compiler::{EntryState, MEMORY_MAIN_ENTRY},
    error::{prelude::*, TypstSourceDiagnostic},
    error_once,
    foundations::Content,
    typst::prelude::*,
    Bytes, TypstDocument, TypstFileId,
};

use crate::{error::NodeTypstCompileResult, map_node_error, CompileDocumentOptions, NodeError};

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
    pub fn setup_compiler_by(
        &mut self,
        compile_by: CompileDocumentOptions,
    ) -> napi::Result<Option<EntryState>, NodeError> {
        let world = self.0.universe_mut();
        let new_state = {
            if let Some(main_file_content) = compile_by.main_file_content {
                if compile_by.main_file_path.is_some() {
                    return Err(map_node_error(error_once!(
                        "main file content and path cannot be specified at the same time"
                    )));
                }

                let new_entry = world.entry_state().select_in_workspace(*MEMORY_MAIN_ENTRY);

                let content = Bytes::from(main_file_content.as_bytes());
                if let Err(err) = world.map_shadow_by_id(*MEMORY_MAIN_ENTRY, content) {
                    return Err(map_node_error(error_once!("cannot map shadow", err: err)));
                }

                Some(new_entry)
            } else if let Some(main_file_path) = compile_by.main_file_path {
                if compile_by.main_file_content.is_some() {
                    return Err(map_node_error(error_once!(
                        "main file content and path cannot be specified at the same time"
                    )));
                }

                let fp = std::path::Path::new(main_file_path.as_str());
                Some(match world.workspace_root() {
                    Some(root) => {
                        if let Ok(p) = root.strip_prefix(fp) {
                            EntryState::new_rooted(
                                root.clone(),
                                Some(TypstFileId::new(
                                    None,
                                    typst_ts_core::typst::syntax::VirtualPath::new(p),
                                )),
                            )
                        } else {
                            EntryState::new_rootless(fp.into()).unwrap()
                        }
                    }
                    None => {
                        return Err(map_node_error(error_once!(
                            "workspace root is not set, cannot set entry file"
                        )))
                    }
                })
            } else {
                None
            }
        };

        let Some(new_state) = new_state else {
            return Ok(None);
        };

        world
            .mutate_entry(new_state)
            .map(Some)
            .map_err(map_node_error)
    }

    pub fn compile_raw(
        &mut self,
        compile_by: CompileDocumentOptions,
    ) -> napi::Result<NodeTypstCompileResult, NodeError> {
        let e = self.setup_compiler_by(compile_by)?;

        let res = self.0.compile(&mut CompileEnv::default()).into();

        if let Some(entry_file) = e {
            self.0
                .universe_mut()
                .mutate_entry(entry_file)
                .map_err(map_node_error)?;
        }

        Ok(res)
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
