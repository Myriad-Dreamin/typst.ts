use std::path::PathBuf;

pub fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fuzzers/corpora")
}

pub fn artifact_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/typst-artifacts")
}

pub fn package_renderer_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/renderer")
}

#[cfg(feature = "web_artifacts")]
pub mod web_artifact;

pub mod std_artifact;
