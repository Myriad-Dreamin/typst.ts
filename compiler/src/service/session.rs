use std::path::PathBuf;

use log::{error, info};
use typst_ts_core::config::CompileOpts;

use crate::world::WorldSnapshot;

#[derive(Default)]
pub struct CompileSession {
    workspace_dir: PathBuf,
    entry_file_path: PathBuf,
    world: Option<crate::TypstSystemWorld>,
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

        self.world = crate::TypstSystemWorld::new(compile_opts)
            .map_err(|err| {
                error!("failed to create world: {:?}", err);
                err
            })
            .ok();
        true
    }

    pub fn take_snapshot(&mut self) -> Option<WorldSnapshot> {
        let begin = instant::Instant::now();
        let world = self.world.as_mut().unwrap();

        world.reset();
        // world.main = world
        //     .resolve(&self.entry_file_path)
        //     .map_err(|err| {
        //         error!("failed to resolve entry file: {:?}", err);
        //         err
        //     })
        //     .ok()?;
        info!("broken take_snapshot");
        info!("take_snapshot resolved in {:?}", begin.elapsed());

        None

        // let mut tracer = Tracer::default();
        // let doc = match typst::compile(world, &mut tracer) {
        //     Ok(doc) => doc,
        //     Err(err) => {
        //         error!("failed to compile: {:?}", err);
        //         return None;
        //     }
        // };
        // info!("take_snapshot compiled in {:?}", begin.elapsed());

        // let font_profile_begin = instant::Instant::now();
        // let font_profile = world.font_resolver.profile().clone();
        // let font_profile_elapsed = font_profile_begin.elapsed();

        // let dependencies_begin = instant::Instant::now();
        // let dependencies = world.get_dependencies();
        // let dependencies_elapsed = dependencies_begin.elapsed();

        // let artifact_begin = instant::Instant::now();
        // let ir = typst_ts_core::artifact_ir::Artifact::from(&doc);
        // let artifact_data = to_base64(ir.get_buffer());
        // let artifact_header = ArtifactHeader {
        //     metadata: ir.metadata,
        //     pages: ir.pages,
        //     offsets: ir.offsets,
        // };
        // let artifact_elapsed = artifact_begin.elapsed();

        // let snapshot = Some(WorldSnapshot {
        //     font_profile: Some(font_profile),
        //     dependencies,

        //     artifact_header,
        //     artifact_data,
        // });

        // info!(
        //     "take_snapshot packed in {:?}:
        // font_profile/dependencies/artifact_elasped = {:?}/{:?}/{:?}",
        //     begin.elapsed(),
        //     font_profile_elapsed,
        //     dependencies_elapsed,
        //     artifact_elapsed
        // );
        // snapshot
    }
}
