use std::path::Path;

use typst::{doc::Document, eval::Tracer};
use typst_ts_compiler::{service::CompileDriver, TypstSystemWorld};
use typst_ts_core::{config::CompileOpts, exporter_builtins::GroupExporter};

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) -> CompileDriver {
    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: workspace_dir.to_owned(),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
        exporter,
    }
}

pub fn test_compiler(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) {
    let mut driver = get_driver(workspace_dir, entry_file_path, exporter);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    for i in 0..200 {
        println!("Iteration {}", i);
        // reset the world caches
        driver.world.reset();

        content.push_str(" user edit");

        // checkout the entry file
        let main_id = driver.main_id();
        driver.world.main = main_id;
        // early error cannot use map_err
        driver
            .world
            .resolve_with(&driver.entry_file, main_id, &content)
            .unwrap();

        // compile and export document
        let mut tracer = Tracer::default();
        typst::compile(&driver.world, &mut tracer).unwrap();
        comemo::evict(10);
    }
}
