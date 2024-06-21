use std::path::{Path, PathBuf};
use std::{borrow::Cow, sync::Arc};

use crate::AsCowBytes;
use reflexo::{error::prelude::*, ImmutPath};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use typst::{
    foundations::Dict,
    syntax::{FileId, VirtualPath},
};

#[derive(Debug, Clone, PartialEq, Eq)]
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
    Detached,
}

impl Default for EntryState {
    fn default() -> Self {
        Self::new_detached()
    }
}

pub static DETACHED_ENTRY: once_cell::sync::Lazy<FileId> = once_cell::sync::Lazy::new(|| {
    FileId::new(None, VirtualPath::new(Path::new("/__detached.typ")))
});

pub static MEMORY_MAIN_ENTRY: once_cell::sync::Lazy<FileId> =
    once_cell::sync::Lazy::new(|| FileId::new(None, VirtualPath::new(Path::new("/__main__.typ"))));

impl EntryState {
    pub fn new_detached() -> Self {
        Self::Detached
    }

    pub fn new_workspace(root: ImmutPath) -> Self {
        Self::Workspace { root, main: None }
    }

    pub fn new_rooted(root: ImmutPath, main: Option<FileId>) -> Self {
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
        match self {
            Self::Workspace { main, .. } => *main,
            Self::PreparedEntry { main, .. } => Some(*main),
            Self::Detached => None,
        }
    }

    pub fn root(&self) -> Option<Arc<Path>> {
        match self {
            Self::Detached => None,
            Self::PreparedEntry { root, .. } => root.clone(),
            Self::Workspace { root, .. } => Some(root.clone()),
        }
    }

    pub fn workspace_root(&self) -> Option<Arc<Path>> {
        match self {
            Self::Detached | Self::PreparedEntry { .. } => None,
            Self::Workspace { root, .. } => Some(root.clone()),
        }
    }

    pub fn select_in_workspace(&self, id: FileId) -> EntryState {
        match self {
            Self::Detached | Self::PreparedEntry { .. } => Self::PreparedEntry {
                entry: id.vpath().as_rooted_path().into(),
                root: None,
                main: id,
            },
            Self::Workspace { root, .. } => Self::Workspace {
                root: root.clone(),
                main: Some(id),
            },
        }
    }

    pub fn try_select_path_in_workspace(
        &self,
        p: &Path,
        allow_rootless: bool,
    ) -> ZResult<Option<EntryState>> {
        Ok(match self.workspace_root() {
            Some(root) => match p.strip_prefix(&root) {
                Ok(p) => Some(EntryState::new_rooted(
                    root.clone(),
                    Some(FileId::new(None, VirtualPath::new(p))),
                )),
                Err(e) => {
                    return Err(
                        error_once!("entry file is not in workspace", err: e, entry: p.display(), root: root.display()),
                    )
                }
            },
            None if allow_rootless => EntryState::new_rootless(p.into()),
            None => None,
        })
    }

    pub fn is_detached(&self) -> bool {
        matches!(self, Self::Detached)
    }

    pub fn is_inactive(&self) -> bool {
        matches!(
            self,
            EntryState::Detached | EntryState::Workspace { main: None, .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntryOpts {
    Workspace {
        /// Path to the root directory of compilation.
        /// The world forbids direct access to files outside this directory.
        root: PathBuf,
        /// Relative path to the main file in the workspace.
        entry: Option<PathBuf>,
    },
    PreparedEntry {
        /// Path to the entry file of compilation.
        entry: PathBuf,
        /// Parent directory of the entry file.
        root: Option<PathBuf>,
    },
    Detached,
}

impl Default for EntryOpts {
    fn default() -> Self {
        Self::Detached
    }
}

impl EntryOpts {
    pub fn new_detached() -> Self {
        Self::Detached
    }

    pub fn new_workspace(root: PathBuf) -> Self {
        Self::Workspace { root, entry: None }
    }

    pub fn new_rooted(root: PathBuf, entry: Option<PathBuf>) -> Self {
        Self::Workspace { root, entry }
    }

    pub fn new_rootless(entry: PathBuf) -> Option<Self> {
        if entry.is_relative() {
            return None;
        }

        Some(Self::PreparedEntry {
            entry: entry.clone(),
            root: entry.parent().map(From::from),
        })
    }
}

impl TryFrom<EntryOpts> for EntryState {
    type Error = reflexo::Error;

    fn try_from(value: EntryOpts) -> Result<Self, Self::Error> {
        match value {
            EntryOpts::Workspace { root, entry } => Ok(EntryState::new_rooted(
                root.as_path().into(),
                entry.map(|e| FileId::new(None, VirtualPath::new(e))),
            )),
            EntryOpts::PreparedEntry { entry, root } => {
                if entry.is_relative() {
                    return Err(error_once!("entry path must be absolute", path: entry.display()));
                }

                // todo: is there path that has no parent?
                let root = root
                    .as_deref()
                    .or_else(|| entry.parent())
                    .ok_or_else(|| error_once!("a root must be determined for EntryOpts::PreparedEntry", path: entry.display()))?;

                let relative_entry = match entry.strip_prefix(root) {
                    Ok(e) => e,
                    Err(_) => {
                        return Err(
                            error_once!("entry path must be inside the root", path: entry.display()),
                        )
                    }
                };

                Ok(EntryState::PreparedEntry {
                    entry: entry.as_path().into(),
                    root: Some(root.into()),
                    main: FileId::new(None, VirtualPath::new(relative_entry)),
                })
            }
            EntryOpts::Detached => Ok(EntryState::new_detached()),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CompileOpts {
    /// Path to entry
    pub entry: EntryOpts,

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
