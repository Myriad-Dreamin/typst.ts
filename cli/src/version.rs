use std::{fmt::Display, process::exit};

use clap::ValueEnum;
use typst_ts_core::build_info::VERSION;

/// Available version formats for `$program -VV`
#[derive(ValueEnum, Debug, Clone)]
#[value(rename_all = "kebab-case")]
pub enum VersionFormat {
    None,
    Short,
    Features,
    Full,
    Json,
    JsonPlain,
}

impl Display for VersionFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.to_possible_value().unwrap().get_name())
    }
}

/// Version information
#[derive(serde::Serialize, serde::Deserialize)]
struct VersionInfo {
    name: &'static str,
    version: &'static str,
    features: Vec<&'static str>,

    program_semver: &'static str,
    program_commit_hash: &'static str,
    program_target_triple: &'static str,
    program_opt_level: &'static str,
    program_build_timestamp: &'static str,

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
            features: env!("VERGEN_CARGO_FEATURES").split(',').collect::<Vec<_>>(),

            program_semver: env!("VERGEN_GIT_DESCRIBE"),
            program_commit_hash: env!("VERGEN_GIT_SHA"),
            program_target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
            program_opt_level: env!("VERGEN_CARGO_OPT_LEVEL"),
            program_build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),

            rustc_semver: env!("VERGEN_RUSTC_SEMVER"),
            rustc_commit_hash: env!("VERGEN_RUSTC_COMMIT_HASH"),
            rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
            rustc_channel: env!("VERGEN_RUSTC_CHANNEL"),
            rustc_llvm_version: env!("VERGEN_RUSTC_LLVM_VERSION"),
        }
    }

    fn program_build(&self) -> String {
        format!(
            "{} with opt_level({}) at {}",
            self.program_target_triple, self.program_opt_level, self.program_build_timestamp
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

/// Print version information and exit if `-VV` is present
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

fn print_full_version(vi: VersionInfo) {
    let program_semver = vi.program_semver;
    let program_commit_hash = vi.program_commit_hash;
    let program_build = vi.program_build();

    let rustc_semver = vi.rustc_semver;
    let rustc_commit_hash = vi.rustc_commit_hash;
    let rustc_build = vi.rustc_build();

    print_short_version(vi);
    println!(
        r##"
program-ver: {program_semver}
program-rev: {program_commit_hash}
program-build: {program_build}

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
