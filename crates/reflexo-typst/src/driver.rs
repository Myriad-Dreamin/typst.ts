use std::{path::Path, sync::Arc};

use reflexo::typst::TypstDocument;
use typst::{
    diag::{eco_format, At, EcoString, FileResult, SourceResult, Warned},
    foundations::Content,
    syntax::Span,
};

use crate::{
    query::retrieve,
    world::{
        CompilerFeat, CompilerUniverse, CompilerWorld, EntryReader, ShadowApi, DETACHED_ENTRY,
    },
};
use crate::{
    vfs::{FileId, PathResolution},
    DynComputation,
};
use crate::{Bytes, TypstFileId, TypstPagedDocument};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<F: CompilerFeat> {
    /// World that has access to the file system.
    pub universe: CompilerUniverse<F>,
}

impl<F: CompilerFeat> CompileDriverImpl<F> {
    pub fn entry_file(&self) -> Option<PathResolution> {
        self.universe.path_for_id(self.universe.main_id()?).ok()
    }
}

impl<F: CompilerFeat> CompileDriverImpl<F> {
    /// Create a new driver.
    pub fn new(_c: DynComputation<F>, universe: CompilerUniverse<F>) -> Self {
        Self { universe }
    }

    pub fn query(
        &mut self,
        selector: String,
        document: &TypstDocument,
    ) -> SourceResult<Vec<Content>> {
        retrieve(&self.universe.snapshot(), &selector, document).at(Span::detached())
    }

    pub fn compile(&mut self) -> SourceResult<Warned<Arc<TypstPagedDocument>>> {
        self.universe().computation().compile()
    }
}

impl<F: CompilerFeat> CompileDriverImpl<F> {
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
}

impl<F: CompilerFeat> ShadowApi for CompileDriverImpl<F> {
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
