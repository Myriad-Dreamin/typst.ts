use log::info;
use typst_ts_dev_server::CompileOpts;

use std::{path::PathBuf, process::exit, sync::Mutex};

use clap::Parser;
use typst_ts_dev_server::{CompileCorpusArgs, CompileSubCommands, Opts, Subcommands};

static COMPILER_PATH: Mutex<Option<String>> = Mutex::new(None);

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Compile(compile_opts) => {
            find_compiler_path(&compile_opts);
            match compile_opts.sub {
                CompileSubCommands::Corpus(args) => compile_corpus(args),
            }
        }
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn find_program_path(dir: &str, program: &str) -> Option<String> {
    let program = PathBuf::from(dir).join(program);
    if program.exists() {
        return Some(program.to_str().unwrap().to_string());
    } else if program.with_extension("exe").exists() {
        return Some(program.with_extension("exe").to_str().unwrap().to_string());
    }
    None
}

fn find_compiler_path(compile_opts: &CompileOpts) {
    const COMPILER_NAME: &str = "typst-ts-cli";

    let mut compiler_path = COMPILER_PATH.lock().unwrap();

    if !compile_opts.compiler.is_empty() {
        let compiler = compile_opts.compiler.clone();
        match compiler.as_str() {
            "debug" => {
                *compiler_path = find_program_path("target/debug", COMPILER_NAME);
            }
            "release" => {
                *compiler_path = find_program_path("target/release", COMPILER_NAME);
            }
            _ => {
                let path = PathBuf::from(&compiler);
                *compiler_path = find_program_path(
                    path.parent().unwrap().to_str().unwrap(),
                    path.file_name().unwrap().to_str().unwrap(),
                );
            }
        }
    } else {
        if compiler_path.is_none() {
            *compiler_path = find_program_path(".", COMPILER_NAME);
        }

        if compiler_path.is_none() {
            *compiler_path = find_program_path("target/debug", COMPILER_NAME);
        }

        if compiler_path.is_none() {
            *compiler_path = find_program_path("target/release", COMPILER_NAME);
        }
    }

    if compiler_path.is_none() {
        eprintln!(
            "Cannot find typst-ts-cli in current directory, target/debug, or target/release."
        );
        exit(1);
    }
    info!("using compiler path: {}", compiler_path.clone().unwrap());
}

fn compile_corpus(args: CompileCorpusArgs) {
    let mut compile_formats = args.format.clone();
    if compile_formats.is_empty() {
        compile_formats.push("ir".to_owned());
        compile_formats.push("json".to_owned());
    }

    let compiler_path = COMPILER_PATH.lock().unwrap();
    let compiler_path = compiler_path.clone().unwrap();
    info!("compile corpus in {}...", args.name);

    let corpus_path = "fuzzers/corpora";

    // get all corpus in workspace_path
    let workspace_path = PathBuf::from(corpus_path).join(&args.name);

    let corpora = std::fs::read_dir(&workspace_path).unwrap();

    for corpus in corpora {
        let corpus_name = corpus.unwrap().file_name();
        if !corpus_name.to_string_lossy().ends_with(".typ") {
            continue;
        }
        info!("compile corpus: {}", corpus_name.to_string_lossy());
        let mut cmd = std::process::Command::new(&compiler_path);
        cmd.arg("compile");
        cmd.arg("--workspace");
        cmd.arg(workspace_path.to_str().unwrap());
        cmd.arg("--entry");
        let entry_path = PathBuf::from(corpus_path)
            .join(&args.name)
            .join(corpus_name);
        cmd.arg(entry_path.to_str().unwrap());

        for compile_format in &compile_formats {
            cmd.arg("--format");
            cmd.arg(compile_format);
        }

        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());

        let status = cmd.status().unwrap();

        if status.code().unwrap() != 0 {
            eprintln!("compile corpus failed.");
            exit(status.code().unwrap());
        }
    }
    exit(0);
}
