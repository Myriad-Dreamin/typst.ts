use std::path::Path;

use reflexo_typst::config::{entry::EntryOpts, CompileOpts};
use reflexo_typst::exporter_builtins::GroupExporter;
use reflexo_typst::{
    CompileDriver, CompileExporter, PureCompiler, ShadowApiExt, TypstDocument, TypstSystemUniverse,
    TypstSystemWorld,
};

fn get_driver(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<TypstDocument>,
) -> CompileDriver<CompileExporter<PureCompiler<TypstSystemWorld>>> {
    let world = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    let world = world.with_entry_file(entry_file_path.to_owned());
    CompileDriver::new(CompileExporter::default().with_exporter(exporter), world)
}

pub fn test_compiler(
    workspace_dir: &Path,
    entry_file_path: &Path,
    exporter: GroupExporter<TypstDocument>,
) {
    let mut driver = get_driver(workspace_dir, entry_file_path, exporter);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    for i in 0..200 {
        eprintln!("Iteration {}", i);

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
