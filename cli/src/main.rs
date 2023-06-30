use std::{
    path::{Path, PathBuf},
    process::exit,
    sync::Arc,
};

use clap::{Args, Command, FromArgMatches};
use typst::{diag::SourceResult, font::FontVariant, World};

use typst_ts_cli::{
    font::EMBEDDED_FONT,
    tracing::TraceGuard,
    utils::{self, UnwrapOrExit},
    version::intercept_version,
    CompileArgs, CompletionArgs, EnvKey, FontSubCommands, ListFontsArgs, MeasureFontsArgs, Opts,
    Subcommands,
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
    human_panic::setup_panic!();

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
            with_embedded_fonts: EMBEDDED_FONT.to_owned(),
            ..CompileOpts::default()
        })
        .unwrap_or_exit();

        let exporter = typst_ts_cli::export::prepare_exporters(&args, entry_file_path);

        CompileDriver {
            world,
            entry_file: entry_file_path.to_owned(),
            exporter,
        }
    };

    #[allow(clippy::type_complexity)]
    let compile_once: Box<dyn Fn(&mut CompileDriver) -> SourceResult<()>> = if args.dynamic_layout {
        Box::new(|driver: &mut CompileDriver| {
            let output_dir = {
                // If output is specified, use it.
                let dir = (!args.output.is_empty()).then(|| Path::new(&args.output));
                // Otherwise, use the parent directory of the entry file.
                let dir = dir.unwrap_or_else(|| {
                    driver
                        .entry_file
                        .parent()
                        .expect("entry_file has no parent")
                });
                dir.join(
                    driver
                        .entry_file
                        .file_name()
                        .expect("entry_file has no file name"),
                )
            };
            CompileDriver::once_dynamic(driver, &output_dir)
        })
    } else {
        Box::new(|driver: &mut CompileDriver| {
            let doc = Arc::new(driver.compile()?);
            driver.export(doc)
        })
    };

    if args.watch {
        utils::async_continue(async move {
            let mut driver = compile_driver();
            typst_ts_cli::watch::watch_dir(workspace_dir, move |events| {
                // relevance checking
                if events.is_some() && !events.unwrap().iter().any(|event| driver.relevant(event)) {
                    return;
                }

                // compile
                driver.with_compile_diag::<true, _>(&compile_once);
                comemo::evict(30);
            })
            .await;
        })
    } else {
        let compiled = compile_driver().with_compile_diag::<false, _>(compile_once);
        utils::logical_exit(compiled.is_some());
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
        with_embedded_fonts: EMBEDDED_FONT.to_owned(),
        ..CompileOpts::default()
    })
    .unwrap_or_exit();

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
    })
    .unwrap_or_exit();

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
