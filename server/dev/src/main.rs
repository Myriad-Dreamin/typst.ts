use std::borrow::Cow;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::exit;

use clap::Parser;
use log::info;
use reflexo_typst::path::PathClean;
use reflexo_typst::{DiagnosticsTask, EntryReader, TaskInputs};
use tokio::io::AsyncBufReadExt;
use typst_ts_cli::export::ReflexoTaskBuilder;
use typst_ts_dev_server::{
    http::run_http, utils::async_continue, CompileCorpusArgs, CompileSubCommands, Opts,
    RunSubCommands, Subcommands, WatchSubCommands,
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
    let corpus_path = current_dir().unwrap().join(corpus_path);

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
            extra_embedded_fonts: typst_dev_assets::fonts().map(Cow::Borrowed).collect(),
            ..Default::default()
        },
        format: compile_formats.clone(),
        ..Default::default()
    };

    let mut verse = typst_ts_cli::compile::resolve_universe(compile_args.compile.clone());
    verse.features = reflexo_typst::Features::from_iter([
        reflexo_typst::Feature::Html,
        reflexo_typst::Feature::A11yExtras,
    ]);
    verse.increment_revision(|this| this.set_creation_timestamp(Some(0)));

    let compile = |cat: &str, name: &str| -> Option<String> {
        let entry = corpus_path.join(cat).join(name).clean();
        let expected_artifacts = expected_artifact_paths(&entry, &compile_formats);
        for artifact in &expected_artifacts {
            let _ = std::fs::remove_file(artifact);
        }

        let mut tb = ReflexoTaskBuilder::new();
        tb.print_compile_status(true);
        tb.args(&compile_args, Some(&entry));
        let exporter = tb.build();

        let entry = verse.entry_state().try_select_path_in_workspace(&entry);
        let graph = verse.computation_with(TaskInputs {
            entry: entry.unwrap(),
            inputs: None,
        });

        let mut errors = vec![];
        if let Err(err) = (exporter)(&graph) {
            errors.push(format!("export failed: {err}"));
        }

        match graph.compute::<DiagnosticsTask>() {
            Ok(diag) if diag.error_cnt() > 0 => {
                errors.push(format!("{} compile error(s)", diag.error_cnt()));
            }
            Err(err) => {
                errors.push(format!("diagnostics failed: {err}"));
            }
            _ => {}
        }

        let missing_artifacts = expected_artifacts
            .iter()
            .filter(|artifact| !artifact.exists())
            .map(|artifact| artifact.display().to_string())
            .collect::<Vec<_>>();
        if !missing_artifacts.is_empty() {
            errors.push(format!(
                "missing artifact(s): {}",
                missing_artifacts.join(", ")
            ));
        }

        (!errors.is_empty()).then(|| format!("{cat}/{name}: {}", errors.join("; ")))
    };

    // get all corpus in workspace_path

    let mut failures = vec![];
    for cat in args.categories.clone() {
        info!("compile corpus in {cat}...");

        let cat_dir = corpus_path.join(&cat);

        for corpus in walkdir::WalkDir::new(&cat_dir)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
        {
            if !corpus.file_type().is_file() {
                continue;
            }

            let path = corpus.path();
            if path.extension() != Some(std::ffi::OsStr::new("typ")) {
                continue;
            }

            match std::fs::read_to_string(path) {
                Ok(text) if text.starts_with("// SKIP") => continue,
                Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
                    info!("skip non-utf8 corpus: {cat:10} {}", path.display());
                    continue;
                }
                _ => {}
            }

            let name = path
                .strip_prefix(&cat_dir)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            info!("compile corpus: {cat:10} {name}");

            if let Some(failure) = compile(&cat, &name) {
                failures.push(failure);
            }
        }
    }

    if !failures.is_empty() {
        eprintln!("compile corpus failed for {} file(s):", failures.len());
        for failure in failures.iter().take(50) {
            eprintln!("  - {failure}");
        }
        if failures.len() > 50 {
            eprintln!("  - ... and {} more", failures.len() - 50);
        }
        exit(1);
    }

    exit(0);
}

fn expected_artifact_paths(entry: &Path, formats: &[String]) -> Vec<PathBuf> {
    let mut paths = formats
        .iter()
        .filter_map(|format| match format.as_str() {
            "ast" => Some(entry.with_extension("ast.ansi.text")),
            "pdf" => Some(entry.with_extension("pdf")),
            "svg" => Some(entry.with_extension("artifact.svg")),
            "svg_html" => Some(entry.with_extension("artifact.svg.html")),
            "sir" | "vector" => Some(entry.with_extension("artifact.sir.in")),
            "text" => Some(entry.with_extension("txt")),
            "nothing" => None,
            _ => None,
        })
        .collect::<Vec<_>>();

    paths.sort();
    paths.dedup();
    paths
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
    let serve_http_cmd = "yarn dev:run";
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
        log::info!("spawn group: {grp}");
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

                            eprintln!("{grp}: {line}");
                        }
                        Ok(line) = stderr.next_line() => {
                            let Some(line) = line else {
                                continue;
                            };

                            eprintln!("{grp}: {line}");
                        }
                        else => {
                            return Ok(());
                        }
                    }
                }
            }

            let _ = watch_stream(child, grp).await;
            eprintln!("{grp}: exited");
            std::process::exit(0);
        }));
    }

    let _ = tokio::signal::ctrl_c().await;
    info!("Ctrl-C received, exiting");
    std::process::exit(0);
}
