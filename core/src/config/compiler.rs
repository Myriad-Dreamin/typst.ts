use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileOpts {
    /// The root directory for compilation routine.
    #[serde(rename = "rootDir")]
    pub root_dir: PathBuf,

    /// Path to entry
    pub entry: PathBuf,

    /// will remove later
    #[serde(rename = "fontPaths")]
    pub font_paths: Vec<PathBuf>,

    /// Exclude system font paths
    #[serde(rename = "noSystemFonts")]
    pub no_system_fonts: bool,
}
