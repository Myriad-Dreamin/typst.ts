#![deny(clippy::all)]

use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};
use typst_ts_compiler::{
    service::{CompileEnv, Compiler, EntryManager},
    ShadowApi, TypstSystemWorld,
};
use typst_ts_core::{
    config::compiler::{EntryState, MEMORY_MAIN_ENTRY},
    error::{prelude::*, TypstFileError, TypstSourceDiagnostic},
    error_once,
    foundations::Content,
    typst::prelude::*,
    Bytes, ImmutPath, TypstDocument, TypstFileId,
};

use crate::{error::NodeTypstCompileResult, map_node_error, CompileDocumentOptions, NodeError};

pub trait NodeCompilerTrait: Compiler<World = TypstSystemWorld> + ShadowApi {
    fn setup_compiler_by(
        &mut self,
        compile_by: CompileDocumentOptions,
    ) -> napi::Result<Option<EntryState>, NodeError> {
        let new_state = {
            if let Some(main_file_content) = compile_by.main_file_content {
                if compile_by.main_file_path.is_some() {
                    return Err(map_node_error(error_once!(
                        "main file content and path cannot be specified at the same time"
                    )));
                }

                let new_entry = self.world().entry.select_in_workspace(*MEMORY_MAIN_ENTRY);

                let content = Bytes::from(main_file_content.as_bytes());
                if let Err(err) = self.world().map_shadow_by_id(*MEMORY_MAIN_ENTRY, content) {
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
                Some(match self.world().workspace_root() {
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

        self.world_mut()
            .mutate_entry(new_state)
            .map(Some)
            .map_err(map_node_error)
    }

    fn compile_raw(
        &mut self,
        compile_by: CompileDocumentOptions,
    ) -> napi::Result<NodeTypstCompileResult, NodeError> {
        let e = self.setup_compiler_by(compile_by)?;

        let res = self.pure_compile(&mut CompileEnv::default()).into();

        if let Some(entry_file) = e {
            self.world_mut()
                .mutate_entry(entry_file)
                .map_err(map_node_error)?;
        }

        Ok(res)
    }
}

pub struct BoxedCompiler(Box<dyn NodeCompilerTrait<World = TypstSystemWorld>>);

impl Deref for BoxedCompiler {
    type Target = dyn NodeCompilerTrait<World = TypstSystemWorld>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for BoxedCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl<T: NodeCompilerTrait<World = TypstSystemWorld> + 'static> From<T> for BoxedCompiler {
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

type SourceResult<T> = Result<T, EcoVec<TypstSourceDiagnostic>>;

type FileResult<T> = Result<T, TypstFileError>;

/// A blanket implementation for all `CompileMiddleware`.
/// If you want to wrap a compiler, you should override methods in
/// `CompileMiddleware`.
impl Compiler for BoxedCompiler {
    type World = TypstSystemWorld;

    #[inline]
    fn world(&self) -> &Self::World {
        self.0.world()
    }

    #[inline]
    fn world_mut(&mut self) -> &mut Self::World {
        self.0.world_mut()
    }

    #[inline]
    fn main_id(&self) -> TypstFileId {
        self.0.main_id()
    }

    #[inline]
    fn reset(&mut self) -> SourceResult<()> {
        self.0.reset()
    }

    #[inline]
    fn pure_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<TypstDocument>> {
        self.0.pure_compile(env)
    }

    #[inline]
    fn pure_query(
        &mut self,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        self.0.pure_query(selector, document)
    }

    #[inline]
    fn compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<TypstDocument>> {
        self.0.compile(env)
    }

    #[inline]
    fn query(&mut self, selector: String, document: &TypstDocument) -> SourceResult<Vec<Content>> {
        self.0.query(selector, document)
    }

    #[inline]
    fn iter_dependencies<'a>(
        &'a self,
        f: &mut dyn FnMut(&'a ImmutPath, FileResult<&typst_ts_compiler::Time>),
    ) {
        self.0.iter_dependencies(f)
    }

    #[inline]
    fn notify_fs_event(&mut self, event: typst_ts_compiler::vfs::notify::FilesystemEvent) {
        self.0.notify_fs_event(event)
    }
}

impl ShadowApi for BoxedCompiler {
    #[inline]
    fn _shadow_map_id(&self, _file_id: TypstFileId) -> FileResult<PathBuf> {
        self.0._shadow_map_id(_file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.0.shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.0.reset_shadow()
    }

    #[inline]
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.0.map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&self, path: &Path) -> FileResult<()> {
        self.0.unmap_shadow(path)
    }
}
