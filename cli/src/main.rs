use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use log::error;
use typst::{font::FontVariant, World};

use typst_ts_cli::{
    compile::CompileAction, diag::Status, tracing::TraceGuard, CompileArgs, FontSubCommands,
    ListFontsArgs, Opts, Subcommands,
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

fn compatible_to_tracing(args: &CompileArgs) -> bool {
    if args.watch {
        error!("cannot use --trace with --watch");
        return false;
    }

    true
}

fn compile(args: CompileArgs) -> ! {
    let workspace_dir = Path::new(args.workspace.as_str());
    let entry_file_path = Path::new(args.entry.as_str());

    let _guard = args.trace.clone().and_then(|t| {
        if !compatible_to_tracing(&args) {
            exit(1);
        }

        TraceGuard::new(t)
            .map_err(|err| {
                error!("init trace failed: {err}");
                exit(1);
            })
            .ok()
    });

    let compile_action = || {
        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: workspace_dir.to_owned(),
            font_paths: args.font_paths.clone(),
            ..CompileOpts::default()
        });

        let exporter = typst_ts_cli::export::prepare_exporters(&args, entry_file_path);

        CompileAction {
            world,
            entry_file: entry_file_path.to_owned(),
            exporter,
        }
    };

    if args.watch {
        typst_ts_cli::utils::async_continue(async {
            let mut compile_action = compile_action();
            typst_ts_cli::watch::watch_dir(workspace_dir, |events| {
                compile_once_watch(entry_file_path, &mut compile_action, events)
            })
            .await;
        })
    } else {
        compile_once(compile_action())
    }

    fn compile_once(mut compile_action: CompileAction) -> ! {
        let compile_result: Result<(), Box<Vec<typst::diag::SourceError>>> = compile_action.once();
        let no_errors = compile_result.is_ok();

        compile_result
            .map_err(|errs| compile_action.print_diagnostics(*errs))
            .unwrap();
        exit(if no_errors { 0 } else { 1 });
    }

    fn compile_once_watch(
        entry_file_path: &Path,
        compile_action: &mut CompileAction,
        events: Option<Vec<notify::Event>>,
    ) {
        // relevance checking
        if events.is_some()
            && !events
                .unwrap()
                .iter()
                .any(|event| compile_action.relevant(&event))
        {
            return;
        }

        // compile
        typst_ts_cli::diag::status(entry_file_path, Status::Compiling).unwrap();
        match compile_action.once() {
            Ok(_) => {
                typst_ts_cli::diag::status(entry_file_path, Status::Success).unwrap();
            }
            Err(errs) => {
                typst_ts_cli::diag::status(entry_file_path, Status::Error).unwrap();
                compile_action.print_diagnostics(*errs).unwrap();
            }
        }
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
