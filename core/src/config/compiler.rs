use std::path::{Path, PathBuf};
use std::{borrow::Cow, sync::Arc};

use crate::AsCowBytes;
use reflexo::ImmutPath;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typst::syntax::Source;
use typst::{
    foundations::Dict,
    syntax::{FileId, VirtualPath},
};

#[derive(Debug, Clone)]
pub enum EntryState {
    Workspace {
        /// Path to the root directory of compilation.
        /// The world forbids direct access to files outside this directory.
        root: Arc<Path>,
        /// Identifier of the main file in the workspace
        main: Option<FileId>,
    },
    PreparedEntry {
        /// Path to the entry file of compilation.
        entry: Arc<Path>,
        /// Parent directory of the entry file.
        root: Option<Arc<Path>>,
        /// Identifier of the main file.
        main: FileId,
    },
    Detached {
        /// Path to the root directory of compilation.
        root: Option<Arc<Path>>,
        /// A source that is not associated with any path.
        source: Source,
    },
}

impl Default for EntryState {
    fn default() -> Self {
        Self::new_detached("".to_string(), None)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryOpts {
    Workspace {
        /// Path to the root directory of compilation.
        /// The world forbids direct access to files outside this directory.
        root: PathBuf,
    },
    PreparedEntry {
        /// Path to the entry file of compilation.
        entry: PathBuf,
        /// Parent directory of the entry file.
        root: Option<PathBuf>,
    },
    Detached {
        /// Path to the root directory of compilation.
        root: Option<PathBuf>,
        /// A source that is not associated with any path.
        source: String,
    },
}

impl Default for EntryOpts {
    fn default() -> Self {
        Self::Detached {
            root: None,
            source: "".to_string(),
        }
    }
}

pub static DETACHED_ENTRY: once_cell::sync::Lazy<FileId> = once_cell::sync::Lazy::new(|| {
    FileId::new(None, VirtualPath::new(Path::new("/__detached.typ")))
});

impl EntryState {
    pub fn new_detached(src: String, root: Option<ImmutPath>) -> Self {
        Self::Detached {
            root,
            source: Source::new(*DETACHED_ENTRY, src),
        }
    }

    pub fn new_with_root(root: ImmutPath, main: Option<FileId>) -> Self {
        Self::Workspace { root, main }
    }

    pub fn new_rootless(entry: ImmutPath) -> Option<Self> {
        Some(Self::PreparedEntry {
            entry: entry.clone(),
            root: entry.parent().map(From::from),
            main: FileId::new(None, VirtualPath::new(entry.file_name()?)),
        })
    }

    pub fn main(&self) -> Option<FileId> {
        Some(match self {
            Self::Workspace { main, .. } => return *main,
            Self::PreparedEntry { main, .. } => *main,
            Self::Detached { source, .. } => source.id(),
        })
    }

    pub fn root(&self) -> Option<Arc<Path>> {
        match self {
            Self::Detached { root, .. } | Self::PreparedEntry { root, .. } => root.clone(),
            Self::Workspace { root, .. } => Some(root.clone()),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileOpts {
    /// The root directory for compilation routine.
    #[serde(rename = "rootDir")]
    pub root_dir: PathBuf,

    /// Path to entry
    pub entry: PathBuf,

    /// Additional input arguments to compile the entry file.
    pub inputs: Dict,

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

#[serde_as]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileFontOpts {
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

impl From<CompileOpts> for CompileFontOpts {
    fn from(opts: CompileOpts) -> Self {
        Self {
            font_profile_cache_path: opts.font_profile_cache_path,
            font_paths: opts.font_paths,
            no_system_fonts: opts.no_system_fonts,
            with_embedded_fonts: opts.with_embedded_fonts,
        }
    }
}
