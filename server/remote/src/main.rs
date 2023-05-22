use std::process::exit;

use clap::Parser;
use typst_ts_remote_server::{Opts, RunArgs, Subcommands};

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let opts = Opts::parse();

    match opts.sub {
        Subcommands::Run(args) => run(args),
    };

    #[allow(unreachable_code)]
    {
        unreachable!("The subcommand must exit the process.");
    }
}

fn run(_args: RunArgs) {
    exit(0);
}
