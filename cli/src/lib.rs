use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(name = "forge", version = "0.1.0")]
pub struct Opts {
    #[clap(subcommand)]
    pub sub: Subcommands,
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "The cli for typst.ts.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommands {
    #[clap(visible_alias = "c", about = "Run precompiler.")]
    Compile(CompileArgs),
}

#[derive(Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Path to typst workspace.
    #[clap(long, short, default_value = ".")]
    pub workspace: String,

    /// Entry file.
    #[clap(long, short, required = true)]
    pub entry: String,

    /// Output formats.
    #[clap(long)]
    pub format: Vec<String>,

    /// Output to directory, default in the same directory as the entry file.
    #[clap(long, short, default_value = "")]
    pub output: String,
}
