use std::path::PathBuf;

pub mod export;
pub mod tracing;
pub mod utils;
pub mod version;
pub mod watch;

use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use typst_ts_core::build_info::VERSION;
use version::VersionFormat;

#[derive(Debug, Parser)]
#[clap(name = "typst-ts-cli", version = VERSION)]
pub struct Opts {
    /// Print Version
    #[arg(short = 'V', long, group = "version-dump")]
    pub version: bool,

    /// Print Version in format
    #[arg(long = "VV", alias = "version-fmt", group = "version-dump", default_value_t = VersionFormat::None)]
    pub vv: VersionFormat,

    #[clap(subcommand)]
    pub sub: Option<Subcommands>,
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

    #[clap(about = "Generate shell completion script.")]
    Completion(CompletionArgs),

    #[clap(about = "Dump Client Environment.")]
    Env(EnvArgs),

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
    /// Measure fonts and generate a profile file for compiler
    Measure(MeasureFontsArgs),
}

#[derive(Debug, Clone, Parser)]
#[clap(next_help_heading = "Compile options")]
pub struct CompileArgs {
    /// Path to typst workspace.
    #[clap(long, short, default_value = ".")]
    pub workspace: String,

    /// Watch mode.
    #[clap(long)]
    pub watch: bool,

    /// Generate dynamic layout representation.
    /// Note: this is an experimental feature and will be merged as
    ///   format `dyn-svg` in the future.
    #[clap(long)]
    pub dynamic_layout: bool,

    /// Enable tracing.
    /// Possible usage: --trace=verbosity={0..3}
    ///   where verbosity: {0..3} -> {warning, info, debug, trace}
    ///
    #[clap(long)]
    pub trace: Option<String>,

    /// Entry file.
    #[clap(long, short, required = true)]
    pub entry: String,

    /// Output formats, possible values: `json`, `pdf`, `svg`,
    ///   `json_glyphs`, `ast`, `ir`, and `rmp`.
    #[clap(long)]
    pub format: Vec<String>,

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

/// Measure fonts and generate a profile file for compiler
#[derive(Debug, Clone, Parser)]
pub struct MeasureFontsArgs {
    /// Add additional directories to search for fonts
    #[clap(long = "font-path", value_name = "DIR", action = ArgAction::Append)]
    pub font_paths: Vec<PathBuf>,

    /// Path to output profile file
    #[arg(long, required = true)]
    pub output: PathBuf,

    /// Exclude system font paths
    #[arg(long)]
    pub no_system_fonts: bool,
}

#[derive(ValueEnum, Debug, Clone)]
pub enum EnvKey {
    Features,
}

/// Generate shell completion script.
#[derive(Debug, Clone, Parser)]
pub struct CompletionArgs {
    /// Completion script kind.
    #[clap(value_enum)]
    pub shell: clap_complete::Shell,
}

/// Dump Client Environment.
#[derive(Debug, Clone, Parser)]
pub struct EnvArgs {
    /// The key of environment kind.
    #[clap(value_name = "KEY")]
    pub key: EnvKey,
}
