use std::path::Path;

use reflexo_typst::config::{entry::EntryOpts, CompileOpts};
use reflexo_typst::{Bytes, TypstSystemUniverse};

fn get_driver(workspace_dir: &Path, entry_file_path: &Path) -> TypstSystemUniverse {
    let verse = TypstSystemUniverse::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.into()),
        no_system_fonts: true,
        ..CompileOpts::default()
    })
    .unwrap();

    verse.with_entry_file(entry_file_path.to_owned())
}

pub fn test_compiler(workspace_dir: &Path, entry_file_path: &Path) {
    let driver = get_driver(workspace_dir, entry_file_path);
    let mut content = { std::fs::read_to_string(entry_file_path).expect("Could not read file") };

    for i in 0..200 {
        eprintln!("Iteration {i}");

        content.push_str(" user edit");

        // checkout the entry file
        driver
            .snapshot_with_entry_content(Bytes::from_string(content.clone()), None)
            .compile()
            .output
            .unwrap();

        comemo::evict(10);
    }
}
