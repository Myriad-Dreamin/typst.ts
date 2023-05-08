use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use log::error;
use typst::{font::FontVariant, World};

use typst_ts_cli::{
    compile::CompileDriver, tracing::TraceGuard, CompileArgs, FontSubCommands, ListFontsArgs, Opts,
    Subcommands,
};
use typst_ts_compiler::TypstSystemWorld;
use typst_ts_core::config::CompileOpts;

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

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
    if args.trace.is_some() && args.watch {
        error!("cannot use --trace with --watch");
        exit(1);
    }

    let workspace_dir = Path::new(args.workspace.as_str());
    let entry_file_path = Path::new(args.entry.as_str());

    let _guard = args
        .trace
        .clone()
        .map(TraceGuard::new)
        .transpose()
        .map_err(|err| {
            error!("init trace failed: {err}");
            exit(1);
        })
        .ok()
        .flatten();

    let compile_driver = || {
        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: workspace_dir.to_owned(),
            font_paths: args.font_paths.clone(),
            ..CompileOpts::default()
        });

        let exporter = typst_ts_cli::export::prepare_exporters(&args, entry_file_path);

        CompileDriver {
            world,
            entry_file: entry_file_path.to_owned(),
            exporter,
        }
    };

    if args.watch {
        typst_ts_cli::utils::async_continue(async {
            let mut driver = compile_driver();
            typst_ts_cli::watch::watch_dir(workspace_dir, |events| {
                compile_once_watch(&mut driver, events)
            })
            .await;
        })
    } else {
        let compiled = compile_driver().once_diag::<false>();
        exit(if compiled { 0 } else { 1 });
    }

    fn compile_once_watch(driver: &mut CompileDriver, events: Option<Vec<notify::Event>>) {
        // relevance checking
        if events.is_some() && !events.unwrap().iter().any(|event| driver.relevant(event)) {
            return;
        }

        // compile
        driver.once_diag::<true>();
        comemo::evict(30);
    }
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
