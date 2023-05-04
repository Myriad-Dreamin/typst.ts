use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::Parser;
use log::{error, info};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use typst::{font::FontVariant, World};

use typst_ts_cli::{
    compile::CompileAction, diag::Status, CompileArgs, FontSubCommands, ListFontsArgs, Opts,
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

fn async_continue<F: std::future::Future<Output = ()>>(f: F) -> ! {
    typst_ts_cli::utils::async_run(f);

    #[allow(unreachable_code)]
    {
        unreachable!("The async command must exit the process.");
    }
}

fn compile(args: CompileArgs) -> ! {
    let workspace_dir = Path::new(args.workspace.as_str());
    let entry_file_path = Path::new(args.entry.as_str());

    let compile_action = || {
        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: workspace_dir.to_owned(),
            font_paths: args.font_paths.clone(),
            ..CompileOpts::default()
        });

        let (document_exporters, artifact_exporters, ir_exporter) =
            typst_ts_cli::export::prepare_exporters(args.clone(), entry_file_path);

        CompileAction {
            world,
            entry_file: entry_file_path.to_owned(),
            doc_exporters: document_exporters,
            artifact_exporters,
            ir_artifact_exporter: ir_exporter,
        }
    };

    if args.watch {
        async_continue(async {
            compile_watch(entry_file_path, workspace_dir, compile_action()).await;
        })
    } else {
        compile_once(compile_action())
    }

    fn compile_once(mut compile_action: CompileAction) -> ! {
        let messages = compile_action.once();
        let no_errors = messages.is_empty();

        compile_action.print_diagnostics(messages).unwrap();
        exit(if no_errors { 0 } else { 1 });
    }

    async fn compile_watch(
        entry_file_path: &Path,
        workspace_dir: &Path,
        mut compile_action: CompileAction,
    ) -> ! {
        // Setup file watching.
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, _>| match res {
                Ok(e) => {
                    tx.send(e).unwrap();
                }
                Err(e) => error!("watch error: {:#}", e),
            },
            notify::Config::default(),
        )
        .map_err(|_| "failed to watch directory")
        .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(workspace_dir, RecursiveMode::Recursive)
            .unwrap();

        // Handle events.
        info!("start watching files...");
        loop {
            typst_ts_cli::diag::status(entry_file_path, Status::Compiling).unwrap();
            let messages = compile_action.once();
            if messages.is_empty() {
                typst_ts_cli::diag::status(entry_file_path, Status::Success).unwrap();
            } else {
                typst_ts_cli::diag::status(entry_file_path, Status::Error).unwrap();
            }
            compile_action.print_diagnostics(messages.clone()).unwrap();
            comemo::evict(30);

            loop {
                let mut events = vec![];
                while let Ok(e) =
                    tokio::time::timeout(tokio::time::Duration::from_millis(100), rx.recv()).await
                {
                    events.push(e);
                }

                let recompile = events
                    .into_iter()
                    .flatten()
                    .any(|event| compile_action.relevant(&event));

                if recompile {
                    break;
                }
            }
        }
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
