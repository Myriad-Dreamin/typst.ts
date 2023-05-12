use std::{fmt::Display, process::exit};

use clap::{builder::PossibleValue, ValueEnum};
use typst_ts_core::build_info::VERSION;

#[derive(Debug, Clone)]
pub enum VersionFormat {
    None,
    Short,
    Features,
    Full,
    Json,
    JsonPlain,
}

impl ValueEnum for VersionFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            VersionFormat::None,
            VersionFormat::Short,
            VersionFormat::Features,
            VersionFormat::Full,
            VersionFormat::Json,
            VersionFormat::JsonPlain,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
        Some(match self {
            VersionFormat::None => PossibleValue::new("none"),
            VersionFormat::Short => PossibleValue::new("short"),
            VersionFormat::Features => PossibleValue::new("features"),
            VersionFormat::Full => PossibleValue::new("full"),
            VersionFormat::Json => PossibleValue::new("json"),
            VersionFormat::JsonPlain => PossibleValue::new("json-plain"),
        })
    }
}

impl Display for VersionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionFormat::None => write!(f, "none"),
            VersionFormat::Short => write!(f, "short"),
            VersionFormat::Features => write!(f, "features"),
            VersionFormat::Full => write!(f, "full"),
            VersionFormat::Json => write!(f, "json"),
            VersionFormat::JsonPlain => write!(f, "json-plain"),
        }
    }
}

fn feature_list() -> Vec<&'static str> {
    env!("VERGEN_CARGO_FEATURES").split(',').collect::<Vec<_>>()
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VersionInfo {
    name: &'static str,
    version: &'static str,
    features: Vec<&'static str>,

    cli_semver: &'static str,
    cli_commit_hash: &'static str,
    cli_target_triple: &'static str,
    cli_profile: &'static str,
    cli_build_timestamp: &'static str,

    rustc_semver: &'static str,
    rustc_commit_hash: &'static str,
    rustc_host_triple: &'static str,
    rustc_channel: &'static str,
    rustc_llvm_version: &'static str,
}

impl VersionInfo {
    fn new() -> Self {
        Self {
            // todo: global app name
            name: "typst-ts-cli",
            version: VERSION,
            features: feature_list(),

            cli_semver: env!("VERGEN_GIT_SEMVER"),
            cli_commit_hash: env!("VERGEN_GIT_SHA"),
            cli_target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
            cli_profile: env!("VERGEN_CARGO_PROFILE"),
            cli_build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),

            rustc_semver: env!("VERGEN_RUSTC_SEMVER"),
            rustc_commit_hash: env!("VERGEN_RUSTC_COMMIT_HASH"),
            rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
            rustc_channel: env!("VERGEN_RUSTC_CHANNEL"),
            rustc_llvm_version: env!("VERGEN_RUSTC_LLVM_VERSION"),
        }
    }

    fn cli_build(&self) -> String {
        format!(
            "{} with {} mode at {}",
            self.cli_target_triple, self.cli_profile, self.cli_build_timestamp
        )
    }

    fn rustc_build(&self) -> String {
        format!(
            "{}-{} with LLVM {}",
            self.rustc_host_triple, self.rustc_channel, self.rustc_llvm_version
        )
    }
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self::new()
    }
}

fn print_full_version(vi: VersionInfo) {
    let cli_semver = vi.cli_semver;
    let cli_commit_hash = vi.cli_commit_hash;
    let cli_build = vi.cli_build();

    let rustc_semver = vi.rustc_semver;
    let rustc_commit_hash = vi.rustc_commit_hash;
    let rustc_build = vi.rustc_build();

    print_short_version(vi);
    println!(
        r##"
cli-ver: {cli_semver}
cli-rev: {cli_commit_hash}
cli-build: {cli_build}

rustc-ver: {rustc_semver}
rustc-rev: {rustc_commit_hash}
rustc-build: {rustc_build}"##
    );
}

fn print_short_version(vi: VersionInfo) {
    let name = vi.name;
    let version = vi.version;
    let features = vi
        .features
        .iter()
        .copied()
        .filter(|&s| s != "default" && !s.ends_with("_exporter"))
        .collect::<Vec<_>>()
        .join(" ");

    println!(
        r##"{name} version {version}
features: {features}"##
    );
}

pub fn intercept_version(v: bool, f: VersionFormat) {
    let f = match f {
        VersionFormat::None if v => VersionFormat::Short,
        VersionFormat::None => return,
        _ => f,
    };
    let version_info = VersionInfo::new();
    match f {
        VersionFormat::Full => print_full_version(version_info),
        VersionFormat::Features => println!("{}", version_info.features.join(",")),
        VersionFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&version_info).unwrap())
        }
        VersionFormat::JsonPlain => println!("{}", serde_json::to_string(&version_info).unwrap()),
        _ => print_short_version(version_info),
    }
    exit(0);
}
