use log::info;

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
        Subcommands::Compile(compile_sub) => {
            find_compiler_path();
            match compile_sub {
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

fn find_compiler_path() {
    let mut compiler_path = COMPILER_PATH.lock().unwrap();

    if compiler_path.is_none() {
        *compiler_path = find_program_path(".", "typst-ts-cli");
    }

    if compiler_path.is_none() {
        *compiler_path = find_program_path("target/debug", "typst-ts-cli");
    }

    if compiler_path.is_none() {
        *compiler_path = find_program_path("target/release", "typst-ts-cli");
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
    let compiler_path = COMPILER_PATH.lock().unwrap();
    let compiler_path = compiler_path.clone().unwrap();
    info!("compile corpus {}...", args.name);

    let corpus_path = "fuzzers/corpora";

    let mut cmd = std::process::Command::new(compiler_path);
    cmd.arg("compile");
    cmd.arg("--workspace");
    let workspace_path = PathBuf::from(corpus_path).join(&args.name);
    cmd.arg(workspace_path.to_str().unwrap());
    cmd.arg("--entry");
    let entry_path = PathBuf::from(corpus_path).join(&args.name).join("main.typ");
    cmd.arg(entry_path.to_str().unwrap());

    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());

    let status = cmd.status().unwrap();

    exit(status.code().unwrap());
}
