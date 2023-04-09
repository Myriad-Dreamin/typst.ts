use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use typst_precompiler::TypstSystemWorld;
use typst_ts_cli::{CompileArgs, Opts, Subcommands};
use typst_ts_core::Artifact;

fn main() {
    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Compile(args) => compile(args),
    }
}

fn compile(args: CompileArgs) {
    let mut root_path = PathBuf::new();
    root_path.push(args.workspace);

    let mut world = TypstSystemWorld::new(root_path);
    world.reset();

    let entry_file = args.entry.as_str();
    let entry_file = Path::new(entry_file);
    let content = { std::fs::read_to_string(entry_file).expect("Could not read file") };

    match world.resolve_with(entry_file, &content) {
        Ok(id) => {
            world.main = id;
        }
        Err(e) => {
            panic!("handler compile error {e}")
        }
    }

    let messages: Vec<_> = match typst::compile(&world) {
        Ok(document) => {
            let output_dir = if !args.output.is_empty() {
                Path::new(&args.output)
            } else {
                entry_file.parent().unwrap()
            };
            let mut output_dir = output_dir.to_path_buf();
            output_dir.push("output");

            // output to pdf
            let buffer = typst::export::pdf(&document);
            let output_path = output_dir
                .with_file_name(entry_file.file_name().unwrap())
                .with_extension("pdf");
            std::fs::write(&output_path, buffer).unwrap();

            // output to artifact json
            let artifact = Artifact::from(document);
            let output_path = output_dir
                .with_file_name(entry_file.file_name().unwrap())
                .with_extension("artifact.json");
            std::fs::write(&output_path, serde_json::to_string(&artifact).unwrap()).unwrap();

            vec![]
        }
        Err(errors) => *errors,
    };

    for err in messages.clone().into_iter() {
        println!("compile error: {:?}", err);
    }

    exit(if messages.is_empty() { 0 } else { 1 })
}
