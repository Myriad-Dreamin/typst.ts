pub(crate) mod macros;

pub mod font;
pub mod vfs;
pub mod workspace;
pub mod world;

#[cfg(feature = "system")]
pub mod service;

#[cfg(feature = "system")]
pub(crate) mod system;
#[cfg(feature = "system")]
pub use system::TypstSystemWorld;

// todo: make compiler work in browser
#[cfg(feature = "browser-compile")]
pub(crate) mod browser;
#[cfg(feature = "browser-compile")]
pub use browser::TypstBrowserWorld;

#[cfg(test)]
mod tests {
    use typst_ts_core::{config::CompileOpts, Artifact};

    use super::*;
    use std::path::PathBuf;

    fn artifact_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fuzzers/corpora/math/main.artifact.json")
    }

    fn artifact_output_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fuzzers/corpora/math/main.artifact.pdf")
    }

    #[test]
    fn test_parse_document() {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: root_path,
            ..CompileOpts::default()
        });
        let artifact_path = artifact_path();
        let content = std::fs::read_to_string(artifact_path).unwrap();
        let artifact: Artifact = serde_json::from_str(content.as_str()).unwrap();
        let document = artifact.to_document(&world.font_resolver);
        let buffer = typst::export::pdf(&document);
        std::fs::write(artifact_output_path(), buffer).unwrap();
    }
}
