use clap::Parser;
use log::info;
use std::{path::PathBuf, process::exit};
use tokio::io::AsyncBufReadExt;
use typst_ts_compiler::service::features::WITH_COMPILING_STATUS_FEATURE;

use typst_ts_compiler::service::{
    CompileEnv, CompileExporter, CompileMiddleware, CompileReporter, Compiler, ConsoleDiagReporter,
    FeatureSet,
};
use typst_ts_core::path::PathClean;
use typst_ts_dev_server::{http::run_http, utils::async_continue, RunSubCommands};

use typst_ts_dev_server::{
    CompileCorpusArgs, CompileSubCommands, Opts, Subcommands, WatchSubCommands,
};

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .filter_module("typst", log::LevelFilter::Warn)
        .filter_module("typst_ts", log::LevelFilter::Info)
        .filter_module("tracing::", log::LevelFilter::Off)
        .try_init();

    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Compile(compile_opts) => match compile_opts.sub {
            CompileSubCommands::Corpus(args) => compile_corpus(args),
        },
        Subcommands::Run(run_sub) => match run_sub {
            RunSubCommands::Http(args) => async_continue(async move {
                run_http(args).await;
                exit(0);
            }),
        },
        Subcommands::Watch(watch_sub) => async_continue(async move {
            watch(watch_sub).await;
            exit(0);
        }),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn compile_corpus(args: CompileCorpusArgs) {
    let corpus_path = "fuzzers/corpora";

    let mut compile_formats = args.format.clone();
    if compile_formats.is_empty() {
        compile_formats.push("svg".to_owned());
        compile_formats.push("sir".to_owned());
    }

    let compile_args = typst_ts_cli::CompileArgs {
        compile: typst_ts_cli::CompileOnceArgs {
            font: typst_ts_cli::FontArgs { paths: vec![] },
            workspace: ".".to_owned(),
            entry: "".to_owned(),
            ..Default::default()
        },
        format: compile_formats.clone(),
        ..Default::default()
    };

    let driver = typst_ts_cli::compile::create_driver(compile_args.compile.clone());

    let driver = CompileExporter::new(driver);

    let mut driver = CompileReporter::new(driver);
    driver.set_generic_reporter(ConsoleDiagReporter::default());

    // enable compiling status
    let feat_set = FeatureSet::default().configure(&WITH_COMPILING_STATUS_FEATURE, true);
    let feat_set = std::sync::Arc::new(feat_set);

    let mut compile = |cat: String, name: String| {
        let entry = PathBuf::from(corpus_path).join(cat).join(name).clean();

        let exporter = typst_ts_cli::export::prepare_exporters(&compile_args, &entry);

        let exporter_layer = driver.inner_mut();

        exporter_layer.set_exporter(exporter);
        exporter_layer.inner_mut().set_entry_file(entry);

        let _ = driver.compile(&mut CompileEnv::default().configure_shared(feat_set.clone()));
    };

    // get all corpus in workspace_path

    for cat in args.catergories.clone() {
        info!("compile corpus in {cat}...");

        let cat_dir = PathBuf::from(corpus_path).join(&cat);

        let corpora = std::fs::read_dir(&cat_dir).unwrap();

        for corpus in corpora {
            let corpus_name = corpus.unwrap().file_name();
            if !corpus_name.to_string_lossy().ends_with(".typ") {
                continue;
            }
            info!("compile corpus: {cat:10} {}", corpus_name.to_string_lossy());

            compile(cat.clone(), corpus_name.to_string_lossy().to_string());
        }
    }
    exit(0);
}

const fn yarn_cmd() -> &'static str {
    if cfg!(windows) {
        "yarn.cmd"
    } else {
        "yarn"
    }
}

async fn watch(watch_sub: WatchSubCommands) {
    let watch_renderer_cmd = "yarn workspace @myriaddreamin/typst-ts-renderer watch";
    let watch_renderer_group = ("renderer", watch_renderer_cmd);
    let watch_core_cmd = "yarn workspace @myriaddreamin/typst.ts build:dev";
    let watch_core_group = ("core", watch_core_cmd);
    let serve_http_cmd = "yarn dev";
    let serve_http_group = ("http", serve_http_cmd);

    let mut groups = vec![];

    match watch_sub {
        WatchSubCommands::Renderer => {
            groups.push(watch_renderer_group);
            groups.push(watch_core_group);
            groups.push(serve_http_group);
        }
    }

    let mut children = vec![];
    // todo: color
    for (grp, cmd) in groups {
        log::info!("spawn group: {}", grp);
        let args = cmd.split(' ').collect::<Vec<_>>();
        let mut cmd = tokio::process::Command::new(if args[0] == "yarn" {
            yarn_cmd()
        } else {
            args[0]
        });

        cmd.args(&args[1..])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let child = cmd.spawn().unwrap();

        // redirect stdout and stderr
        // todo color missing
        children.push(tokio::spawn(async move {
            async fn watch_stream(
                child: tokio::process::Child,
                grp: &'static str,
            ) -> std::io::Result<()> {
                let stdout = child.stdout.unwrap();
                let stderr = child.stderr.unwrap();

                let mut stdout = tokio::io::BufReader::new(stdout).lines();
                let mut stderr = tokio::io::BufReader::new(stderr).lines();

                loop {
                    tokio::select! {
                        Ok(line) = stdout.next_line() => {
                            let Some(line) = line else {
                                continue;
                            };

                            println!("{}: {}", grp, line);
                        }
                        Ok(line) = stderr.next_line() => {
                            let Some(line) = line else {
                                continue;
                            };

                            println!("{}: {}", grp, line);
                        }
                        else => {
                            return Ok(());
                        }
                    }
                }
            }

            let _ = watch_stream(child, grp).await;
            println!("{}: exited", grp);
            std::process::exit(0);
        }));
    }

    let _ = tokio::signal::ctrl_c().await;
    info!("Ctrl-C received, exiting");
    std::process::exit(0);
}
