use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    world::{CompilerFeat, CompilerWorld},
    NotifyApi, ShadowApi,
};
use typst::{
    diag::{eco_format, At, EcoString, Hint, SourceResult},
    foundations::Content,
    syntax::Span,
    World,
};
use typst_ts_core::{
    config::compiler::DETACHED_ENTRY, Bytes, ImmutPath, TypstDocument, TypstFileId,
};

use super::{CompileEnv, Compiler, EntryManager, EnvWorld};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<C, W> {
    pub compiler: C,
    /// World that has access to the file system.
    pub world: W,
}

impl<C: Compiler, W: World> CompileDriverImpl<C, W> {
    /// Create a new driver.
    pub fn new(compiler: C, world: W) -> Self {
        Self { compiler, world }
    }

    pub fn query(
        &mut self,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        self.compiler.query(&self.world, selector, document)
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
}

impl<C: Compiler, F: CompilerFeat> CompileDriverImpl<C, CompilerWorld<F>> {
    pub fn entry_file(&self) -> Option<PathBuf> {
        let main = self.world.entry.main()?;
        self.world.path_for_id(main).ok()
    }
}

impl<C: Compiler, W: World + EnvWorld + EntryManager> CompileDriverImpl<C, W> {
    pub fn compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<TypstDocument>> {
        let world = &mut self.world;

        world.prepare_env(env)?;

        let main_id = world
            .main_id()
            .ok_or_else(|| eco_format!("no entry file"))
            .at(Span::detached())?;
        world
            .source(main_id)
            .hint(AtFile(main_id))
            .at(Span::detached())?;

        self.compiler.compile(world, env)
    }
}

impl<C: Compiler, W: World + EnvWorld + EntryManager + NotifyApi> CompileDriverImpl<C, W> {
    pub fn world(&self) -> &W {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut W {
        &mut self.world
    }

    pub fn main_id(&self) -> TypstFileId {
        self.world.main_id().unwrap_or_else(|| *DETACHED_ENTRY)
    }

    /// reset the compilation state
    pub fn reset(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset()?;

        Ok(())
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

    pub fn iter_dependencies(&self, f: &mut dyn FnMut(ImmutPath)) {
        self.world.iter_dependencies(f)
    }

    pub fn notify_fs_event(&mut self, event: crate::vfs::notify::FilesystemEvent) {
        self.world.notify_fs_event(event)
    }
}

impl<C: Compiler, W: World + ShadowApi> ShadowApi for CompileDriverImpl<C, W> {
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
