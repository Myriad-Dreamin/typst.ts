use clap::{Parser, Subcommand};
use typst_ts_core::build_info::VERSION;

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
pub struct RunArgs {}
