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
    about = "The dev-server for typst.ts.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    Compile(CompileOpts),
}

#[derive(Debug, Parser)]
#[clap(
    about = "Commands about compile alias for typst.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub struct CompileOpts {
    #[clap(subcommand)]
    pub sub: CompileSubCommands,

    /// Using underlying compiler to compile corpus.
    #[clap(long, default_value = "")]
    pub compiler: String,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "Commands about compile alias for typst.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum CompileSubCommands {
    /// Compile corpus for typst.ts.
    Corpus(CompileCorpusArgs),
}

#[derive(Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile Corpus options")]
pub struct CompileCorpusArgs {
    /// The name of Corpus.
    #[clap(value_name = "NAME", index = 1)]
    pub name: String,

    /// Output formats.
    #[clap(long)]
    pub format: Vec<String>,
}
