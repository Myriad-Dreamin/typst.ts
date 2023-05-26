use std::path::PathBuf;

use log::error;
use serde::{Deserialize, Serialize};
use typst_ts_core::{config::CompileOpts, font::FontProfile};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceUnit {
    name: String,
    hash: String,
    content: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub font_profile: Option<FontProfile>,
    pub sources: Vec<SourceUnit>,
}

#[derive(Default)]
pub struct CompileSession {
    workspace_dir: PathBuf,
    entry_file_path: PathBuf,
    world: Option<typst_ts_compiler::TypstSystemWorld>,
}

impl CompileSession {
    pub fn initialize(&mut self, entry_file: PathBuf, compile_opts: CompileOpts) -> bool {
        let workspace = compile_opts.root_dir.clone();

        if !entry_file.starts_with(&workspace) {
            error!("invalid entry_file: {}", entry_file.display());
            return false;
        }

        self.workspace_dir = workspace;
        self.entry_file_path = entry_file;

        self.world = Some(typst_ts_compiler::TypstSystemWorld::new(compile_opts));
        true
    }

    pub fn take_snapshot(&mut self) -> Option<WorldSnapshot> {
        let world = self.world.as_mut().unwrap();

        if let Err(err) = typst::compile(world) {
            error!("failed to compile: {:?}", err);
            return None;
        }

        // todo: collect sources
        let mut _sources = Vec::new();

        let font_profile = world.font_resolver.profile().clone();

        Some(WorldSnapshot {
            font_profile: Some(font_profile),
            sources: _sources,
        })
    }
}
