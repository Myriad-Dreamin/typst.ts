#![deny(clippy::all)]

use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::Arc,
};
use typst_ts_compiler::{
    service::{CompileEnv, Compiler, EntryFileState, WorkspaceProvider},
    ShadowApi, TypstSystemWorld,
};
use typst_ts_core::{
    error::{prelude::*, TypstFileError, TypstSourceDiagnostic},
    error_once,
    foundations::Content,
    typst::prelude::*,
    Bytes, ImmutPath, TypstDocument, TypstFileId,
};

use crate::{map_node_error, CompileBy, NodeError, NodeTypstDocument};

pub trait NodeCompilerTrait:
    Compiler<World = TypstSystemWorld> + ShadowApi + EntryFileState
{
    fn setup_compiler_by(
        &mut self,
        compile_by: CompileBy,
    ) -> napi::Result<Option<PathBuf>, NodeError> {
        let e;
        if let Some(main_file_content) = compile_by.main_file_content {
            if compile_by.main_file_path.is_some() {
                return Err(map_node_error(error_once!(
                    "main file content and path cannot be specified at the same time"
                )));
            }

            let generated_file_path = self.world().workspace_root().join("__memory_file__.typ");
            self.map_shadow(
                &generated_file_path,
                Bytes::from(main_file_content.into_bytes()),
            )
            .context("failed to map shadow file")
            .map_err(map_node_error)?;

            e = Some(self.get_entry_file().clone());
            self.set_entry_file(generated_file_path.to_owned())
                .map_err(map_node_error)?;
        } else if let Some(main_file_path) = compile_by.main_file_path {
            if compile_by.main_file_content.is_some() {
                return Err(map_node_error(error_once!(
                    "main file content and path cannot be specified at the same time"
                )));
            }

            e = Some(self.get_entry_file().clone());
            let main_file_path = std::path::Path::new(main_file_path.as_str());
            self.set_entry_file(main_file_path.to_owned())
                .map_err(map_node_error)?;
        } else {
            e = None;
        }

        Ok(e)
    }

    fn compile_raw(&mut self, compile_by: CompileBy) -> napi::Result<NodeTypstDocument, NodeError> {
        let e = self.setup_compiler_by(compile_by)?;

        let res = self
            .pure_compile(&mut CompileEnv::default())
            .map_err(map_node_error);

        if let Some(entry_file) = e {
            self.set_entry_file(entry_file).map_err(map_node_error)?;
        }

        Ok(NodeTypstDocument(res?))
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
    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, typst_ts_compiler::Time)) {
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
