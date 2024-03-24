use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{NotifyApi, ShadowApi};
use typst::{
    diag::{eco_format, At, SourceResult},
    syntax::Span,
    World,
};
use typst_ts_core::{config::compiler::EntryState, Bytes, ImmutPath, TypstFileId};

use super::{Compiler, EntryManager, EnvWorld};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<W: World> {
    /// World that has access to the file system.
    pub world: W,
    /// Path to the entry file.
    entry_file: Arc<Path>,
}

impl<W: World> CompileDriverImpl<W> {
    /// Create a new driver.
    pub fn new(world: W) -> Self {
        Self {
            world,
            entry_file: Path::new("").into(),
        }
    }
}

impl<W: World + EntryManager> CompileDriverImpl<W> {
    /// Wrap driver with a given entry file.
    pub fn with_entry_file(mut self, entry_file: PathBuf) -> Self {
        self.set_entry_file(entry_file.as_path().into()).unwrap();
        self
    }

    /// set an entry file.
    pub fn set_entry_file(&mut self, entry_file: Arc<Path>) -> SourceResult<()> {
        let state = self.world.entry_state();
        let root = matches!(state, EntryState::Workspace { .. })
            .then(|| self.world.workspace_root())
            .flatten();

        self.world
            .mutate_entry(match root {
                Some(root) => match entry_file.strip_prefix(&root) {
                    Ok(p) => EntryState::new_with_root(
                        root.clone(),
                        Some(TypstFileId::new(
                            None,
                            typst_ts_core::typst::syntax::VirtualPath::new(p),
                        )),
                    ),
                    Err(e) => {
                        return Err(eco_format!("entry file is not in workspace: {}", e))
                            .at(Span::detached())
                    }
                },
                None => EntryState::new_rootless(self.entry_file.clone()).unwrap(),
            })
            .map(|_| ())?;
        self.entry_file = entry_file;
        Ok(())
    }

    pub fn entry_file(&self) -> &Path {
        &self.entry_file
    }
}

impl<W: World + EnvWorld + EntryManager + NotifyApi> Compiler for CompileDriverImpl<W> {
    type World = W;

    fn world(&self) -> &Self::World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut Self::World {
        &mut self.world
    }

    fn main_id(&self) -> TypstFileId {
        self.world.main_id().unwrap()
    }

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset()?;

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
