use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub version: String,

    /// Path to typst workspace.
    pub workspace: String,

    /// Path to entries
    pub files: Vec<String>,

    #[serde(rename = "fontPaths")]
    pub font_paths: Vec<String>,
}
