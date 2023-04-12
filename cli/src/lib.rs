use std::path::PathBuf;

pub mod diag;
pub mod utils;

use clap::{ArgAction, Parser, Subcommand};

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
    #[clap(visible_alias = "c", about = "Run compiler.")]
    Compile(CompileArgs),

    #[clap(subcommand)]
    Font(FontSubCommands),
}

#[derive(Debug, Subcommand)]
#[clap(
    about = "Commands about font for typst.",
    after_help = "",
    next_display_order = None
)]
#[allow(clippy::large_enum_variant)]
pub enum FontSubCommands {
    /// List all discovered fonts in system and custom font paths
    List(ListFontsArgs),
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

    /// watch mode.
    #[clap(long)]
    pub watch: bool,
}

/// List all discovered fonts in system and custom font paths
#[derive(Debug, Clone, Parser)]
pub struct ListFontsArgs {
    /// Add additional directories to search for fonts
    #[clap(long = "font-path", value_name = "DIR", action = ArgAction::Append)]
    pub font_paths: Vec<PathBuf>,

    /// Also list style variants of each font family
    #[arg(long)]
    pub variants: bool,
}
