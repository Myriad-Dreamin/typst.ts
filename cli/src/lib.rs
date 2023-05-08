use std::path::PathBuf;

pub mod compile;
pub mod diag;
pub mod export;
pub mod tracing;
pub mod utils;
pub mod watch;

use clap::{ArgAction, Parser, Subcommand};
use typst_ts_core::build_info::VERSION;

#[derive(Debug, Parser)]
#[clap(name = "typst-ts-cli", version = VERSION)]
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

    /// watch mode.
    #[clap(long)]
    pub watch: bool,

    /// enable tracing.
    /// possible usage: --trace=verbosity={0..2}
    /// verbosity:
    /// + 0: warning
    /// + 1: info
    /// + 2: debug
    /// + 3: trace
    ///
    #[clap(long)]
    pub trace: Option<String>,

    /// Entry file.
    #[clap(long, short, required = true)]
    pub entry: String,

    /// Output formats, possible values: `json`, `pdf`, `web_socket`, `ast`, and `rmp`.
    #[clap(long)]
    pub format: Vec<String>,

    /// Output WebSocket subscriber url
    #[clap(long, default_value = "")]
    pub web_socket: String,

    /// Output to directory, default in the same directory as the entry file.
    #[clap(long, short, default_value = "")]
    pub output: String,

    /// Add additional directories to search for fonts
    #[clap(long = "font-path", value_name = "DIR", action = ArgAction::Append)]
    pub font_paths: Vec<PathBuf>,
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
