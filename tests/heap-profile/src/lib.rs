use std::path::Path;

use typst::model::Document;
use typst_ts_compiler::{
    service::{CompileDriver, CompileExporter, Compiler},
    ShadowApiExt, TypstSystemWorld,
};
use typst_ts_core::{config::CompileOpts, exporter_builtins::GroupExporter};

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<Document>,
) -> CompileExporter<CompileDriver> {
    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: workspace_dir.to_owned(),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    let driver = CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
    };

    CompileExporter::new(driver).with_exporter(exporter)
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

        content.push_str(" user edit");

        // checkout the entry file
        let main_id = driver.main_id();

        driver
            .with_shadow_file_by_id(main_id, content.as_bytes().into(), |driver| {
                driver.compile(&mut Default::default())
            })
            .unwrap();

        comemo::evict(10);
    }
}
