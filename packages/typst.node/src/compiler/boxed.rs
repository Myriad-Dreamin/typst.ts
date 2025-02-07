#![deny(clippy::all)]

use std::ops::{Deref, DerefMut};

use reflexo_typst::config::entry::MEMORY_MAIN_ENTRY;
use reflexo_typst::{
    error_once, Bytes, CompileDriver, EntryReader, ShadowApi, TaskInputs, TypstSystemWorld,
};

use super::create_inputs;
use crate::{error::NodeTypstCompileResult, map_node_error, CompileDocArgs, NodeError};

// <World = TypstSystemWorld>
// pub trait NodeCompilerTrait: Compiler
// where
//     Self::W: EntryManager + ShadowApi,
// {
// }

pub struct BoxedCompiler(Box<CompileDriver>);

impl Deref for BoxedCompiler {
    type Target = CompileDriver;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl DerefMut for BoxedCompiler {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}

impl From<CompileDriver> for BoxedCompiler {
    fn from(value: CompileDriver) -> Self {
        Self(Box::new(value))
    }
}

impl BoxedCompiler {
    /// Create a snapshoted world by typst.node's [`CompileDocArgs`].
    /// Should not affect the current universe (global state).
    pub fn create_world(
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
                    .select_in_workspace(MEMORY_MAIN_ENTRY.vpath().as_rooted_path());

                let main_id = new_entry.main().unwrap();

                let content = Bytes::from_string(main_file_content);
                // TODO: eliminate the side effect of shadow mapping safely
                if let Err(err) = self.universe_mut().map_shadow_by_id(main_id, content) {
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
                let fp = abs_fp.as_ref().map(std::path::Path::new).map_err(|e| {
                    map_node_error(error_once!("cannot absolutize the main file path", err: e))
                })?;
                universe
                    .entry_state()
                    .try_select_path_in_workspace(fp)
                    .map_err(map_node_error)?
            } else {
                None
            }
        };

        // Convert the input pairs to a dictionary.
        let inputs = compile_by.inputs.map(create_inputs);

        Ok(self.universe.snapshot_with(Some(TaskInputs {
            entry: new_state,
            inputs,
        })))
    }

    pub fn compile_raw(
        &mut self,
        compile_by: CompileDocArgs,
    ) -> napi::Result<NodeTypstCompileResult, NodeError> {
        let world = self.create_world(compile_by)?;

        if world.entry_state().is_inactive() {
            return Err(map_node_error(error_once!("entry file is not set")));
        }

        let c = self.universe().computation();
        // FIXME: This is implementation detail, use a better way from
        // the compiler driver.
        c.ensure_main().map_err(map_node_error)?;
        Ok(c.compile().into())
    }
}
