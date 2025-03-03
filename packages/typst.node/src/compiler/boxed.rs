#![deny(clippy::all)]

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use reflexo_typst::system::SystemWorldComputeGraph;
use reflexo_typst::{
    error_once, ArcInto, Bytes, CompilationTask, EntryReader, FlagTask, TaskInputs, TypstDocument,
    TypstSystemUniverse,
};

use super::create_inputs;
use crate::error::*;
use crate::NodeTypstDocument;
use crate::{CompileDocArgs, NodeError};

// <World = TypstSystemWorld>
// pub trait NodeCompilerTrait: Compiler
// where
//     Self::W: EntryManager + ShadowApi,
// {
// }

pub struct BoxedCompiler(Box<TypstSystemUniverse>);

impl Deref for BoxedCompiler {
    type Target = TypstSystemUniverse;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for BoxedCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl From<TypstSystemUniverse> for BoxedCompiler {
    fn from(value: TypstSystemUniverse) -> Self {
        Self(Box::new(value))
    }
}

impl BoxedCompiler {
    /// Create a snapshoted world by typst.node's [`CompileDocArgs`].
    /// Should not affect the current universe (global state).
    pub fn computation(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> Result<Arc<SystemWorldComputeGraph>, NodeError> {
        let universe = self.deref_mut();
        // Convert the input pairs to a dictionary.
        let inputs = compile_by.inputs.map(create_inputs);
        if let Some(main_file_content) = compile_by.main_file_content {
            if compile_by.main_file_path.is_some() {
                return Err(error_once!(
                    "main file content and path cannot be specified at the same time"
                ))?;
            }

            return Ok(universe.snapshot_with_entry_content(
                Bytes::from_string(main_file_content.clone()),
                Some(TaskInputs {
                    entry: None,
                    inputs,
                }),
            ));
        };

        let entry = if let Some(main_file_path) = compile_by.main_file_path {
            if compile_by.main_file_content.is_some() {
                return Err(error_once!(
                    "main file content and path cannot be specified at the same time"
                ))?;
            }

            let abs_fp = std::path::absolute(main_file_path.as_str());
            let fp = abs_fp
                .as_ref()
                .map(std::path::Path::new)
                .map_err(|e| error_once!("cannot absolutize the main file path", err: e))?;
            universe.entry_state().try_select_path_in_workspace(fp)?
        } else {
            None
        };

        Ok(universe.computation_with(TaskInputs { entry, inputs }))
    }

    pub fn compile_raw2<
        D: reflexo_typst::TypstDocumentTrait + ArcInto<TypstDocument> + Send + Sync + 'static,
    >(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> Result<ExecResultRepr<NodeTypstDocument>, NodeError> {
        let graph = self.computation(compile_by)?;

        let _ = graph.provide::<FlagTask<CompilationTask<D>>>(Ok(FlagTask::flag(true)));
        let result = graph.compute::<CompilationTask<D>>()?;
        let result: ExecResultRepr<Arc<D>> = result.as_ref().clone().expect("enabled").into();

        Ok(result
            .map(|d| NodeTypstDocument {
                graph: graph.clone(),
                doc: d.arc_into(),
            })
            .with_graph(graph))
    }
}
