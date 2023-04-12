pub(crate) mod model;
pub use model::*;

pub(crate) mod path;

#[cfg(test)]
mod tests {
    use typst_ts_core::Artifact;

    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn test_parse_document() {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let world = TypstSystemWorld::new(root_path);
        let path = Path::new("fuzzers/corpora/hw/main.artifact.json");
        let content = std::fs::read_to_string(path).unwrap();
        let artifact: Artifact = serde_json::from_str(content.as_str()).unwrap();
        let document = artifact.to_document(&world);
        let buffer = typst::export::pdf(&document);
        let output_path = Path::new("fuzzers/corpora/hw/main2.pdf");
        std::fs::write(&output_path, buffer).unwrap();
    }
}
