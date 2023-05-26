use std::path::PathBuf;

use log::error;
use serde::{Deserialize, Serialize};
use typst::World;
use typst_ts_compiler::workspace::dependency::DependencyTree;
use typst_ts_core::{config::CompileOpts, font::FontProfile};

#[derive(Debug, Serialize, Deserialize)]
pub struct SourceUnit {
    name: String,
    hash: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub font_profile: Option<FontProfile>,
    pub dependencies: DependencyTree,
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

        world.reset();
        world.main = world
            .resolve(&self.entry_file_path)
            .map_err(|err| {
                error!("failed to resolve entry file: {:?}", err);
                err
            })
            .ok()?;

        if let Err(err) = typst::compile(world) {
            error!("failed to compile: {:?}", err);
            return None;
        }

        let font_profile = world.font_resolver.profile().clone();

        Some(WorldSnapshot {
            font_profile: Some(font_profile),
            dependencies: world.get_dependencies(),
        })
    }
}
