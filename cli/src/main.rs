use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use typst::{font::FontVariant, World};
use typst_ts_cli::{
    diag::print_diagnostics, CompileArgs, FontSubCommands, ListFontsArgs, Opts, Subcommands,
};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::{config::CompileOpts, Artifact};

fn main() {
    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Compile(args) => compile(args),
        Subcommands::Font(font_sub) => match font_sub {
            FontSubCommands::List(args) => list_fonts(args),
        },
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn compile(args: CompileArgs) -> ! {
    let mut root_path = PathBuf::new();
    root_path.push(args.workspace);

    let mut world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        ..CompileOpts::default()
    });
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

    print_diagnostics(&world, messages.clone()).unwrap();

    exit(if messages.is_empty() { 0 } else { 1 })
}

fn list_fonts(command: ListFontsArgs) -> ! {
    let mut root_path = PathBuf::new();
    // todo: should cover default workspace path
    root_path.push("-");

    let mut world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        font_paths: command.font_paths,
        ..CompileOpts::default()
    });
    world.reset();

    for (name, infos) in world.book().families() {
        println!("{name}");
        if command.variants {
            for info in infos {
                let FontVariant {
                    style,
                    weight,
                    stretch,
                } = info.variant;
                println!("- Style: {style:?}, Weight: {weight:?}, Stretch: {stretch:?}");
            }
        }
    }

    exit(0)
}
