use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;
use typst_ts_core::build_info::VERSION;

pub(crate) mod compile;
pub mod utils;
pub mod ws;

#[derive(Debug, Parser)]
#[clap(name = "typst-ts-dev-server", version = VERSION)]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "The remote-server for typst.ts.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    Run(RunArgs),
}

#[derive(Debug, Clone, Parser)]
#[clap(next_help_heading = "Run options")]
pub struct RunArgs {
    /// The workspace directory.
    #[clap(long)]
    pub root: String,

    /// The web-socket address.
    #[clap(long, default_value = "")]
    pub web_socket: String,

    /// Add additional directories to search for fonts
    #[clap(long = "font-path", value_name = "DIR", action = ArgAction::Append)]
    pub font_paths: Vec<PathBuf>,
}
