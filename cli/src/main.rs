use std::{
    path::{Path, PathBuf},
    process::exit,
};

use clap::{Args, Command, FromArgMatches};
use typst::{font::FontVariant, World};

use typst_ts_cli::{
    tracing::TraceGuard, version::intercept_version, CompileArgs, CompletionArgs, EnvKey,
    FontSubCommands, ListFontsArgs, MeasureFontsArgs, Opts, Subcommands,
};
use typst_ts_compiler::{service::CompileDriver, TypstSystemWorld};
use typst_ts_core::config::CompileOpts;

fn get_cli(sub_command_required: bool) -> Command {
    let cli = Command::new("$").disable_version_flag(true);
    Opts::augment_args(cli).subcommand_required(sub_command_required)
}

fn help_sub_command() {
    Opts::from_arg_matches(&get_cli(true).get_matches()).unwrap();
}

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("typst", log::LevelFilter::Warn)
        .filter_module("typst_library", log::LevelFilter::Warn)
        .try_init();

    let opts = Opts::from_arg_matches(&get_cli(false).get_matches())
        .map_err(|err| err.exit())
        .unwrap();

    intercept_version(opts.version, opts.vv);

    match opts.sub {
        Some(Subcommands::Compile(args)) => compile(args),
        Some(Subcommands::Completion(args)) => generate_completion(args),
        Some(Subcommands::Env(args)) => match args.key {
            EnvKey::Features => {
                intercept_version(false, typst_ts_cli::version::VersionFormat::Features)
            }
        },
        Some(Subcommands::Font(font_sub)) => match font_sub {
            FontSubCommands::List(args) => list_fonts(args),
            FontSubCommands::Measure(args) => measure_fonts(args),
        },
        None => help_sub_command(),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn compile(args: CompileArgs) -> ! {
    if args.trace.is_some() && args.watch {
        clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            "cannot use option \"--trace\" and \"--watch\" at the same time\n",
        )
        .exit()
    }

    let workspace_dir = Path::new(args.workspace.as_str());
    let entry_file_path = Path::new(args.entry.as_str());

    let _trace_guard = {
        let guard = args.trace.clone().map(TraceGuard::new);
        let guard = guard.transpose().map_err(|err| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidValue,
                format!("init trace failed: {err}\n"),
            )
            .exit()
        });
        guard.unwrap()
    };

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

fn generate_completion(CompletionArgs { shell }: CompletionArgs) -> ! {
    clap_complete::generate(
        shell,
        &mut get_cli(true),
        "typst-ts-cli",
        &mut std::io::stdout(),
    );
    exit(0);
}

fn list_fonts(command: ListFontsArgs) -> ! {
    let mut root_path = PathBuf::new();
    // todo: should cover default workspace path
    root_path.push("-");

    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        font_paths: command.font_paths,
        ..CompileOpts::default()
    });

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

fn measure_fonts(args: MeasureFontsArgs) -> ! {
    let mut root_path = PathBuf::new();
    // todo: should cover default workspace path
    root_path.push("-");

    let mut font_profile_paths = vec![];
    if args.output.exists() {
        font_profile_paths.push(args.output.clone());
    }

    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: root_path,
        font_profile_paths,
        font_paths: args.font_paths,
        font_profile_cache_path: args.output.clone(),
        no_system_fonts: args.no_system_fonts,
        ..CompileOpts::default()
    });

    // create directory for args.output
    if let Some(output) = args.output.parent() {
        std::fs::create_dir_all(output).unwrap();
    }

    let profile = serde_json::to_vec(world.font_resolver.profile()).unwrap();

    // gzip
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&profile).unwrap();
    std::fs::write(args.output, encoder.finish().unwrap()).unwrap();

    exit(0)
}
