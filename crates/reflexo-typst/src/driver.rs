use std::{path::Path, sync::Arc};

use typst::{
    diag::{eco_format, EcoString, FileResult, SourceResult, Warned},
    foundations::Content,
};

use super::{CompileEnv, Compiler};
use crate::vfs::{FileId, PathResolution};
use crate::world::{
    CompilerFeat, CompilerUniverse, CompilerWorld, EntryReader, ShadowApi, DETACHED_ENTRY,
};
use crate::{Bytes, TypstDocument, TypstFileId};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<C, F: CompilerFeat> {
    pub compiler: C,
    /// World that has access to the file system.
    pub universe: CompilerUniverse<F>,
}

impl<C: Compiler, F: CompilerFeat> CompileDriverImpl<C, F> {
    pub fn entry_file(&self) -> Option<PathResolution> {
        self.universe.path_for_id(self.main_id()).ok()
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> CompileDriverImpl<C, F> {
    /// Create a new driver.
    pub fn new(compiler: C, universe: CompilerUniverse<F>) -> Self {
        Self { compiler, universe }
    }

    pub fn query(
        &mut self,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        self.compiler.query(&self.snapshot(), selector, document)
    }

    pub fn compile(&mut self, env: &mut CompileEnv) -> SourceResult<Warned<Arc<TypstDocument>>> {
        let world = self.snapshot();
        self.compiler.ensure_main(&world)?;
        self.compiler.compile(&world, env)
    }
}

impl<C: Compiler, F: CompilerFeat> CompileDriverImpl<C, F> {
    pub fn universe(&self) -> &CompilerUniverse<F> {
        &self.universe
    }

    pub fn universe_mut(&mut self) -> &mut CompilerUniverse<F> {
        &mut self.universe
    }

    pub fn snapshot(&self) -> CompilerWorld<F> {
        self.universe.snapshot()
    }

    pub fn main_id(&self) -> TypstFileId {
        self.universe.main_id().unwrap_or_else(|| *DETACHED_ENTRY)
    }

    /// reset the compilation state
    pub fn reset(&mut self) -> SourceResult<()> {
        self.universe.reset();
        Ok(())
    }

    /// evict the compilation state
    pub fn evict(&mut self, vfs_threshold: usize) -> SourceResult<()> {
        // evict the world caches
        self.universe.evict(vfs_threshold);

        Ok(())
    }

    /// The default implementation of `relevant` method, which performs a
    /// simple check on the event kind.
    /// It returns following values:
    /// - `Some(true)`: the event must be relevant to the compiler.
    /// - `Some(false)`: the event must not be relevant to the compiler.
    /// - `None`: the event may be relevant to the compiler.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    fn _relevant(&self, event: &notify::Event) -> Option<bool> {
        use notify::event::ModifyKind;
        use notify::EventKind;

        macro_rules! fs_event_must_relevant {
            () => {
                // create a file in workspace
                EventKind::Create(_) |
                // rename a file in workspace
                EventKind::Modify(ModifyKind::Name(_))
            };
        }
        macro_rules! fs_event_may_relevant {
            () => {
                // remove/modify file in workspace
                EventKind::Remove(_) | EventKind::Modify(ModifyKind::Data(_)) |
                // unknown manipulation in workspace
                EventKind::Any | EventKind::Modify(ModifyKind::Any)
            };
        }
        macro_rules! fs_event_never_relevant {
            () => {
                // read/write meta event
                EventKind::Access(_) | EventKind::Modify(ModifyKind::Metadata(_)) |
                // `::notify` internal events other event that we cannot identify
                EventKind::Other | EventKind::Modify(ModifyKind::Other)
            };
        }

        return match &event.kind {
            fs_event_must_relevant!() => Some(true),
            fs_event_may_relevant!() => None,
            fs_event_never_relevant!() => Some(false),
        };

        // assert that all cases are covered
        const _: () = match EventKind::Any {
            fs_event_must_relevant!() | fs_event_may_relevant!() | fs_event_never_relevant!() => {}
        };
    }
    /// Check whether a file system event is relevant to the world.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    pub fn relevant(&self, event: &notify::Event) -> bool {
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
}

impl<C: Compiler, F: CompilerFeat> ShadowApi for CompileDriverImpl<C, F> {
    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.universe.shadow_paths()
    }

    #[inline]
    fn shadow_ids(&self) -> Vec<TypstFileId> {
        self.universe.shadow_ids()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.universe.reset_shadow()
    }

    #[inline]
    fn map_shadow(&mut self, path: &Path, content: Bytes) -> FileResult<()> {
        self.universe.map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&mut self, path: &Path) -> FileResult<()> {
        self.universe.unmap_shadow(path)
    }

    #[inline]
    fn map_shadow_by_id(&mut self, file_id: FileId, content: Bytes) -> FileResult<()> {
        self.universe.map_shadow_by_id(file_id, content)
    }

    #[inline]
    fn unmap_shadow_by_id(&mut self, file_id: FileId) -> FileResult<()> {
        self.universe.unmap_shadow_by_id(file_id)
    }
}

struct AtFile(TypstFileId);

impl From<AtFile> for EcoString {
    fn from(at: AtFile) -> Self {
        eco_format!("at file {:?}", at.0)
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
