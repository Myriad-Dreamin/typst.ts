use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileOpts {
    /// The root directory for compilation routine.
    #[serde(rename = "rootDir")]
    pub root_dir: PathBuf,

    /// Path to entry
    pub entry: PathBuf,

    /// Path to font profiles
    #[serde(rename = "fontProfilePaths")]
    pub font_profile_paths: Vec<PathBuf>,

    /// Path to font profile for cache
    #[serde(rename = "fontProfileCachePath")]
    pub font_profile_cache_path: PathBuf,

    /// will remove later
    #[serde(rename = "fontPaths")]
    pub font_paths: Vec<PathBuf>,

    /// Exclude system font paths
    #[serde(rename = "noSystemFonts")]
    pub no_system_fonts: bool,
}
