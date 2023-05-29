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
#[cfg(target_arch = "wasm32")]
pub const MAIN_ARTIFACT_JSON: &[u8] =
    include_bytes!("../../../fuzzers/corpora/math/main.artifact.json");

#[cfg(feature = "web_artifacts")]
#[cfg(target_arch = "wasm32")]
pub const MAIN_ARTIFACT_IR: &[u8] =
    include_bytes!("../../../fuzzers/corpora/math/main.artifact.tir.bin");
