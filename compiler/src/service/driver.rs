use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{NotifyApi, ShadowApi};
use typst::{diag::SourceResult, syntax::VirtualPath, World};
use typst_ts_core::{path::PathClean, Bytes, ImmutPath, TypstFileId};

use super::{Compiler, WorkspaceProvider};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<W: World> {
    /// World that has access to the file system.
    pub world: W,
    /// Path to the entry file.
    pub entry_file: PathBuf,
}

impl<W: World> CompileDriverImpl<W> {
    /// Create a new driver.
    pub fn new(world: W) -> Self {
        Self {
            world,
            entry_file: PathBuf::default(),
        }
    }

    /// Wrap driver with a given entry file.
    pub fn with_entry_file(mut self, entry_file: PathBuf) -> Self {
        self.entry_file = entry_file;
        self
    }

    /// set an entry file.
    pub fn set_entry_file(&mut self, entry_file: PathBuf) {
        self.entry_file = entry_file;
    }
}

impl<W: World + WorkspaceProvider> CompileDriverImpl<W> {
    /// Get the file id for a given path.
    /// Note: only works for files in the workspace instead of external
    /// packages.
    pub fn id_for_path(&self, pb: PathBuf) -> TypstFileId {
        let pb = if pb.is_absolute() {
            let pb = pb.clean();
            let root = self.world.workspace_root().clean();
            pb.strip_prefix(root).unwrap().to_owned()
        } else {
            pb
        };
        TypstFileId::new(None, VirtualPath::new(pb))
    }
}

impl<W: World + WorkspaceProvider + NotifyApi> Compiler for CompileDriverImpl<W> {
    type World = W;

    fn world(&self) -> &Self::World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut Self::World {
        &mut self.world
    }

    fn main_id(&self) -> TypstFileId {
        self.id_for_path(self.entry_file.clone())
    }

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset()?;
        // checkout the entry file
        self.world.set_main_id(self.main_id());

        Ok(())
    }

    /// Check whether a file system event is relevant to the world.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    fn relevant(&self, event: &notify::Event) -> bool {
        // todo: remove this check
        if event
            .paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".artifact."))
        {
            return false;
        }

        self._relevant(event).unwrap_or(true)
    }

    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, crate::Time)) {
        self.world.iter_dependencies(f)
    }

    fn notify_fs_event(&mut self, event: crate::vfs::notify::FilesystemEvent) {
        self.world.notify_fs_event(event)
    }
}

impl<W: World + ShadowApi> ShadowApi for CompileDriverImpl<W> {
    #[inline]
    fn _shadow_map_id(&self, file_id: TypstFileId) -> typst::diag::FileResult<PathBuf> {
        self.world._shadow_map_id(file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.world.shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.world.reset_shadow()
    }

    #[inline]
    fn map_shadow(&self, path: &Path, content: Bytes) -> typst::diag::FileResult<()> {
        self.world.map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&self, path: &Path) -> typst::diag::FileResult<()> {
        self.world.unmap_shadow(path)
    }
}

// todo: Print that a package downloading is happening.
// fn print_downloading(_spec: &PackageSpec) -> std::io::Result<()> {
// let mut w = color_stream();
// let styles = term::Styles::default();

// w.set_color(&styles.header_help)?;
// write!(w, "downloading")?;

// w.reset()?;
// writeln!(w, " {spec}")
// Ok(())
// }
