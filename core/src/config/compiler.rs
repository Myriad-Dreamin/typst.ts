use std::borrow::Cow;
use std::path::PathBuf;

use crate::AsCowBytes;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileOpts {
    /// The root directory for compilation routine.
    #[serde(rename = "rootDir")]
    pub root_dir: PathBuf,

    /// Path to entry
    pub entry: PathBuf,

    /// Path to font profile for cache
    #[serde(rename = "fontProfileCachePath")]
    pub font_profile_cache_path: PathBuf,

    /// will remove later
    #[serde(rename = "fontPaths")]
    pub font_paths: Vec<PathBuf>,

    /// Exclude system font paths
    #[serde(rename = "noSystemFonts")]
    pub no_system_fonts: bool,

    /// Include embedded fonts
    #[serde(rename = "withEmbeddedFonts")]
    #[serde_as(as = "Vec<AsCowBytes>")]
    pub with_embedded_fonts: Vec<Cow<'static, [u8]>>,
}
