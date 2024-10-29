use core::fmt;
use std::{
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, OnceLock},
};

use chrono::{DateTime, Datelike, Local};
use parking_lot::{Mutex, RwLock};
use reflexo::QueryRef;
use reflexo::{error::prelude::*, hash::FxDashMap};
use reflexo::{hash::FxHashMap, ImmutPath};
use reflexo_vfs::{notify::FilesystemEvent, FileId, Vfs};
use reflexo_vfs::{FsProvider, TypstFileId};
use typst::{
    diag::{eco_format, At, EcoString, FileError, FileResult, SourceResult},
    foundations::{Bytes, Datetime, Dict},
    syntax::{Source, Span},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, World,
};

use crate::{
    entry::{EntryManager, EntryReader, EntryState, DETACHED_ENTRY},
    font::FontResolver,
    package::{PackageRegistry, PackageSpec},
    parser::{
        get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
        SemanticTokensLegend,
    },
    CodespanError, CodespanResult, CompilerFeat, ShadowApi, WorldDeps,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct TaskInputs {
    pub entry: Option<EntryState>,
    pub inputs: Option<Arc<LazyHash<Dict>>>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct TaskState {
    pub entry: EntryState,
    pub inputs: Arc<LazyHash<Dict>>,
}

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Revision {
    pub value: Arc<NonZeroUsize>,
}

impl Revision {
    fn acquire(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }

    fn increment(&self) -> Self {
        Self {
            value: Arc::new(self.value.checked_add(1).unwrap()),
        }
    }
}

pub trait Revised {
    fn last_accessed_rev(&self) -> NonZeroUsize;
}

pub struct Revising<'a, T> {
    pub revision: NonZeroUsize,
    pub inner: &'a mut T,
}

impl<'a, T> std::ops::Deref for Revising<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<'a, T> std::ops::DerefMut for Revising<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<'a, F: CompilerFeat> Revising<'a, CompilerUniverse<F>> {
    pub fn vfs(&mut self) -> &mut Vfs<F::AccessModel> {
        &mut self.inner.vfs
    }

    /// Let the vfs notify the access model with a filesystem event.
    ///
    /// See `reflexo_vfs::NotifyAccessModel` for more information.
    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.inner.vfs.notify_fs_event(event);
    }

    pub fn reset_shadow(&mut self) {
        self.inner.vfs.reset_shadow()
    }

    pub fn map_shadow(&mut self, path: &Path, content: Bytes) -> FileResult<()> {
        self.inner.vfs.map_shadow(path, content)
    }

    pub fn unmap_shadow(&mut self, path: &Path) -> FileResult<()> {
        self.inner.vfs.remove_shadow(path);
        Ok(())
    }
}

/// A universe that provides access to the physical system.
///
/// Use [`CompilerUniverse::new`] to create a new universe.
/// Use [`CompilerUniverse::snapshot`] to create a new world.
// #[derive(Debug)]
pub struct CompilerUniverse<F: CompilerFeat> {
    /// The base state.
    base: WorldState<F>,
}

impl<F: CompilerFeat> Deref for CompilerUniverse<F> {
    type Target = WorldState<F>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<F: CompilerFeat> DerefMut for CompilerUniverse<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

/// Creates, snapshots, and manages the compiler universe.
impl<F: CompilerFeat> CompilerUniverse<F> {
    /// Create a [`CompilerUniverse`] with feature implementation.
    ///
    /// Although this function is public, it is always unstable and not intended
    /// to be used directly.
    /// + See [`crate::TypstSystemUniverse::new`] for system environment.
    /// + See [`crate::TypstBrowserUniverse::new`] for browser environment.
    pub fn new_raw(
        entry: EntryState,
        inputs: Option<Arc<LazyHash<Dict>>>,
        vfs: Vfs<F::AccessModel>,
        registry: F::Registry,
        font_resolver: Arc<F::FontResolver>,
    ) -> Self {
        Self {
            base: WorldState {
                revision: Revision {
                    value: Arc::new(NonZeroUsize::new(1).expect("initial revision is 1")),
                },
                base: Arc::new(SharedWorldState {
                    rev_lock: RwLock::new(()),
                    font_resolver,
                    registry: Arc::new(registry),
                    sources: ParseState::default(),
                    task_entries: FxDashMap::default(),
                }),
                task: Arc::new(LazyHash::new(TaskState {
                    entry,
                    inputs: inputs.unwrap_or_default(),
                })),
                vfs,
            },
        }
    }

    /// Increment revision with actions.
    pub fn increment_revision<T>(&mut self, f: impl FnOnce(&mut Revising<Self>) -> T) -> T {
        let base = self.base.base.clone();
        let (prev, next) = {
            let _lg = base.rev_lock.write();
            let prev_revision = self.revision.acquire();
            let revision = self.revision.increment();
            let value = *revision.value;
            self.revision = revision;
            (prev_revision, value)
        };

        let res = f(&mut Revising {
            inner: self,
            revision: next,
        });

        // The order is important here.
        self.base.gc(&prev);
        res
    }

    /// Set the inputs for the compiler.
    pub fn set_inputs(&mut self, inputs: Arc<LazyHash<Dict>>) {
        self.reset();
        Arc::make_mut(&mut self.task).inputs = inputs;
    }

    /// Wrap driver with a given entry file.
    pub fn with_entry_file(mut self, entry_file: PathBuf) -> Self {
        let _ = self.set_entry_file(entry_file.as_path().into());
        self
    }

    /// set an entry file.
    pub fn set_entry_file(&mut self, entry_file: Arc<Path>) -> SourceResult<()> {
        let state = self.entry_state();
        let state = state
            .try_select_path_in_workspace(&entry_file, true)
            .map_err(|e| eco_format!("cannot select entry file out of workspace: {e}"))
            .at(Span::detached())?
            .ok_or_else(|| eco_format!("failed to determine root"))
            .at(Span::detached())?;

        self.mutate_entry(state)?;
        Ok(())
    }

    pub fn do_reparse(&self) -> bool {
        true
    }
}

impl<F: CompilerFeat> CompilerUniverse<F> {
    /// Reset the world for a new lifecycle (of garbage collection).
    pub fn reset(&mut self) {
        self.vfs.reset();
    }

    pub fn get_semantic_token_legend(&self) -> Arc<SemanticTokensLegend> {
        Arc::new(get_semantic_tokens_legend())
    }

    pub fn get_semantic_tokens(
        &self,
        file_path: Option<String>,
        encoding: OffsetEncoding,
    ) -> ZResult<Arc<Vec<SemanticToken>>> {
        let world = match file_path {
            Some(e) => {
                let path = Path::new(&e);
                let s = self
                    .entry_state()
                    .try_select_path_in_workspace(path, true)?
                    .ok_or_else(|| error_once!("cannot select file", path: e))?;

                self.snapshot_with(TaskInputs {
                    entry: Some(s),
                    inputs: None,
                })
            }
            None => self.snapshot(),
        };

        let src = world
            .source(world.main())
            .map_err(|e| error_once!("cannot access source file", err: e))?;
        Ok(Arc::new(get_semantic_tokens_full(&src, encoding)))
    }
}

impl<F: CompilerFeat> ShadowApi for CompilerUniverse<F> {
    #[inline]
    fn _shadow_map_id(&self, file_id: TypstFileId) -> FileResult<PathBuf> {
        self.path_for_id(file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.vfs.shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.increment_revision(|this| this.vfs.reset_shadow())
    }

    #[inline]
    fn map_shadow(&mut self, path: &Path, content: Bytes) -> FileResult<()> {
        self.increment_revision(|this| this.vfs().map_shadow(path, content))
    }

    #[inline]
    fn unmap_shadow(&mut self, path: &Path) -> FileResult<()> {
        self.increment_revision(|this| {
            this.vfs().remove_shadow(path);
            Ok(())
        })
    }
}

impl<F: CompilerFeat> EntryReader for CompilerUniverse<F> {
    fn entry_state(&self) -> EntryState {
        self.entry().clone()
    }
}

impl<F: CompilerFeat> EntryManager for CompilerUniverse<F> {
    fn reset(&mut self) -> SourceResult<()> {
        self.reset();
        Ok(())
    }

    fn mutate_entry(&mut self, state: EntryState) -> SourceResult<EntryState> {
        self.reset();
        Arc::make_mut(&mut self.task).entry = state.clone();
        Ok(state)
    }
}

pub struct CompilerWorld<F: CompilerFeat> {
    /// The base state.
    pub base: WorldState<F>,
    /// Provides library for typst compiler.
    pub library: Arc<LazyHash<Library>>,
    /// The slots for all the files during a single lifecycle.
    pub slots: Arc<Mutex<FxHashMap<TypstFileId, SourceCache>>>,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceLock<DateTime<Local>>,
}

impl<F: CompilerFeat> Deref for CompilerWorld<F> {
    type Target = WorldState<F>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<F: CompilerFeat> DerefMut for CompilerWorld<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl<F: CompilerFeat> CompilerWorld<F> {
    /// Lookup a source file by id.
    #[track_caller]
    fn lookup(&self, id: TypstFileId) -> Source {
        self.source(id)
            .expect("file id does not point to any source file")
    }

    fn map_source_or_default<T>(
        &self,
        id: TypstFileId,
        default_v: T,
        f: impl FnOnce(Source) -> CodespanResult<T>,
    ) -> CodespanResult<T> {
        match World::source(self, id).ok() {
            Some(source) => f(source),
            None => Ok(default_v),
        }
    }

    pub fn take_state(&mut self) {
        std::mem::take(&mut self.slots);
    }

    /// Get all the files that are currently in the VFS.
    ///
    /// This is typically corresponds to the file dependencies of a single
    /// compilation.
    ///
    /// When you don't reset the vfs for each compilation, this function will
    /// still return remaining files from the previous compilation.
    pub fn iter_dependencies_dyn<'a>(
        &'a self,
        p: &'a impl FsProvider,
        f: &mut dyn FnMut(ImmutPath),
    ) {
        for slot in self.slots.lock().iter() {
            f(p.file_path(slot.1.fid));
        }
    }

    /// Insert a new slot into the vfs.
    fn slot<T>(&self, id: TypstFileId, fid: FileId, f: impl FnOnce(&SourceCache) -> T) -> T {
        let mut slots = self.slots.lock();
        f(slots.entry(id).or_insert_with(|| SourceCache {
            fid,
            source: FileQuery::default(),
            buffer: FileQuery::default(),
        }))
    }

    /// Get file content by path.
    pub fn file_inner(
        &self,
        id: TypstFileId,
        fid: FileId,
        p: &impl FsProvider,
    ) -> FileResult<Bytes> {
        self.slot(id, fid, |slot| slot.buffer.compute(|| p.read(fid)).cloned())
    }

    /// Get source content by path and assign the source with a given typst
    /// global file id.
    ///
    /// See `Vfs::resolve_with_f` for more information.
    pub fn parse(&self, id: TypstFileId, fid: FileId, p: &impl FsProvider) -> FileResult<Source> {
        self.slot(id, fid, |slot| {
            slot.source
                .compute(|| {
                    let rev = self.revision();
                    let content = slot.buffer.compute(|| p.read(fid))?.clone();
                    self.sources.entry(rev, id, content).parse().clone()
                })
                .cloned()
        })
    }
}

impl<F: CompilerFeat> ShadowApi for CompilerWorld<F> {
    #[inline]
    fn _shadow_map_id(&self, file_id: TypstFileId) -> FileResult<PathBuf> {
        self.path_for_id(file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.vfs.shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.take_state();
        self.vfs.reset_shadow()
    }

    #[inline]
    fn map_shadow(&mut self, path: &Path, content: Bytes) -> FileResult<()> {
        self.take_state();
        self.vfs.map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&mut self, path: &Path) -> FileResult<()> {
        self.take_state();
        self.vfs.remove_shadow(path);
        Ok(())
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    /// The standard library.
    fn library(&self) -> &LazyHash<Library> {
        self.library.as_ref()
    }

    /// Access the main source file.
    fn main(&self) -> TypstFileId {
        self.entry().main().unwrap_or_else(|| *DETACHED_ENTRY)
    }

    /// Metadata about all known fonts.
    fn font(&self, id: usize) -> Option<Font> {
        self.font_resolver.font(id)
    }

    /// Try to access the specified file.
    fn book(&self) -> &LazyHash<FontBook> {
        self.font_resolver.font_book()
    }

    /// Try to access the specified source file.
    ///
    /// The returned `Source` file's [id](Source::id) does not have to match the
    /// given `id`. Due to symlinks, two different file id's can point to the
    /// same on-disk file. Implementors can deduplicate and return the same
    /// `Source` if they want to, but do not have to.
    fn source(&self, id: TypstFileId) -> FileResult<Source> {
        static DETACH_SOURCE: LazyLock<Source> =
            LazyLock::new(|| Source::new(*DETACHED_ENTRY, String::new()));

        if id == *DETACHED_ENTRY {
            return Ok(DETACH_SOURCE.clone());
        }

        let fid = self.vfs.file_id(&self.path_for_id(id)?);
        self.parse(id, fid, &self.vfs)
    }

    /// Try to access the specified file.
    fn file(&self, id: TypstFileId) -> FileResult<Bytes> {
        let fid = self.vfs.file_id(&self.path_for_id(id)?);
        self.file_inner(id, fid, &self.vfs)
    }

    /// Get the current date.
    ///
    /// If no offset is specified, the local date should be chosen. Otherwise,
    /// the UTC date should be chosen with the corresponding offset in hours.
    ///
    /// If this function returns `None`, Typst's `datetime` function will
    /// return an error.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let now = self.now.get_or_init(chrono::Local::now);

        let naive = match offset {
            None => now.naive_local(),
            Some(o) => now.naive_utc() + chrono::Duration::try_hours(o)?,
        };

        Datetime::from_ymd(
            naive.year(),
            naive.month().try_into().ok()?,
            naive.day().try_into().ok()?,
        )
    }

    /// A list of all available packages and optionally descriptions for them.
    ///
    /// This function is optional to implement. It enhances the user experience
    /// by enabling autocompletion for packages. Details about packages from the
    /// `@preview` namespace are available from
    /// `https://packages.typst.org/preview/index.json`.
    fn packages(&self) -> &[(PackageSpec, Option<EcoString>)] {
        self.registry.packages()
    }
}

impl<F: CompilerFeat> EntryReader for CompilerWorld<F> {
    fn entry_state(&self) -> EntryState {
        self.entry().clone()
    }
}

impl<F: CompilerFeat> WorldDeps for CompilerWorld<F> {
    #[inline]
    fn iter_dependencies(&self, f: &mut dyn FnMut(ImmutPath)) {
        self.iter_dependencies_dyn(&self.vfs, f)
    }
}

impl<'a, F: CompilerFeat> codespan_reporting::files::Files<'a> for CompilerWorld<F> {
    /// A unique identifier for files in the file provider. This will be used
    /// for rendering `diagnostic::Label`s in the corresponding source files.
    type FileId = TypstFileId;

    /// The user-facing name of a file, to be displayed in diagnostics.
    type Name = String;

    /// The source code of a file.
    type Source = Source;

    /// The user-facing name of a file.
    fn name(&'a self, id: TypstFileId) -> CodespanResult<Self::Name> {
        let vpath = id.vpath();
        Ok(if let Some(package) = id.package() {
            format!("{package}{}", vpath.as_rooted_path().display())
        } else {
            match self.entry().root() {
                Some(root) => {
                    // Try to express the path relative to the working directory.
                    vpath
                        .resolve(&root)
                        // differ from typst
                        // .and_then(|abs| pathdiff::diff_paths(&abs, self.workdir()))
                        .as_deref()
                        .unwrap_or_else(|| vpath.as_rootless_path())
                        .to_string_lossy()
                        .into()
                }
                None => vpath.as_rooted_path().display().to_string(),
            }
        })
    }

    /// The source code of a file.
    fn source(&'a self, id: TypstFileId) -> CodespanResult<Self::Source> {
        Ok(self.lookup(id))
    }

    /// See [`codespan_reporting::files::Files::line_index`].
    fn line_index(&'a self, id: TypstFileId, given: usize) -> CodespanResult<usize> {
        let source = self.lookup(id);
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    /// See [`codespan_reporting::files::Files::column_number`].
    fn column_number(&'a self, id: TypstFileId, _: usize, given: usize) -> CodespanResult<usize> {
        let source = self.lookup(id);
        source.byte_to_column(given).ok_or_else(|| {
            let max = source.len_bytes();
            if given <= max {
                CodespanError::InvalidCharBoundary { given }
            } else {
                CodespanError::IndexTooLarge { given, max }
            }
        })
    }

    /// See [`codespan_reporting::files::Files::line_range`].
    fn line_range(
        &'a self,
        id: TypstFileId,
        given: usize,
    ) -> CodespanResult<std::ops::Range<usize>> {
        self.map_source_or_default(id, 0..0, |source| {
            source
                .line_to_range(given)
                .ok_or_else(|| CodespanError::LineTooLarge {
                    given,
                    max: source.len_lines(),
                })
        })
    }
}

pub struct SharedWorldState<F: CompilerFeat> {
    /// Shared source cache.
    pub sources: ParseState,
    /// Provides font management for typst compiler.
    pub font_resolver: Arc<F::FontResolver>,
    /// Provides package management for typst compiler.
    pub registry: Arc<F::Registry>,

    /// The lock for the revision changement.
    rev_lock: RwLock<()>,
    /// The cache entries for each entry
    task_entries: FxDashMap<Arc<LazyHash<TaskState>>, WorldCache<F>>,
}

impl<F: CompilerFeat> SharedWorldState<F> {
    fn create_world(
        self: &Arc<Self>,
        rev: Revision,
        task: Arc<LazyHash<TaskState>>,
        vfs: Vfs<F::AccessModel>,
    ) -> CompilerWorld<F> {
        let mut entry = self.task_entries.entry(task.clone()).or_default();

        let world = entry
            .revisions
            .iter()
            .find_map(|w| (w.revision == *rev.value).then(|| w.clone()));

        let world = world.unwrap_or_else(|| {
            let world = CompilerWorldResource {
                library: create_library(task.inputs.clone()),
                slots: Default::default(),
                base: self.clone(),
                task,
                vfs,
                revision: *rev.value,
                now: OnceLock::new(),
            };
            entry.revisions.push(world.clone());
            world
        });

        world.realize(rev)
    }

    fn gc(self: &Arc<Self>, revision: &Revision) {
        if Arc::strong_count(&revision.value) != 1 {
            return;
        }

        self.do_gc(revision);
    }

    fn do_gc(self: &Arc<Self>, revision: &Revision) {
        let _lg = self.rev_lock.write();
        if Arc::strong_count(&revision.value) != 1 {
            return;
        }

        // Safety: the revision is not shared anymore.
        for mut task_ref in self.task_entries.iter_mut() {
            task_ref.revisions.retain(|w| *revision.value != w.revision);
        }

        drop(_lg);
        self.sources.gc(*revision.value);
    }

    /// Returns the overall memory usage for the stored files.
    pub fn memory_usage(&self) -> usize {
        let mut w = self.sources.cache_entries.len() * core::mem::size_of::<ParseCache>();
        w += self
            .sources
            .cache_entries
            .iter()
            .map(|slot| slot.computes.keys().map(|k| k.len() * 10).sum::<usize>())
            .sum::<usize>();

        w
    }
}

#[derive(Default)]
struct WorldCache<F: CompilerFeat> {
    /// The current active revisions.
    revisions: Vec<CompilerWorldResource<F>>,
}

#[derive(Clone)]
struct CompilerWorldResource<F: CompilerFeat> {
    /// The base state.
    base: Arc<SharedWorldState<F>>,
    /// State for the *root & entry* of compilation.
    /// The world forbids direct access to files outside this directory.
    task: Arc<LazyHash<TaskState>>,
    /// The current revision of the source database.
    revision: NonZeroUsize,
    /// Provides library for typst compiler.
    library: Arc<LazyHash<Library>>,
    /// Provides path-based data access for typst compiler.
    vfs: Vfs<F::AccessModel>,
    /// The slots for all the files during a single lifecycle.
    slots: Arc<Mutex<FxHashMap<TypstFileId, SourceCache>>>,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceLock<DateTime<Local>>,
}

impl<F: CompilerFeat> CompilerWorldResource<F> {
    fn realize(self, revision: Revision) -> CompilerWorld<F> {
        assert_eq!(self.revision, *revision.value, "revision mismatch");
        CompilerWorld::<F> {
            base: WorldState {
                revision,
                base: self.base,
                task: self.task,
                vfs: self.vfs,
            },
            library: self.library,
            slots: self.slots,
            now: self.now,
        }
    }
}

pub struct WorldState<F: CompilerFeat> {
    /// The current revision of the shared state.
    revision: Revision,
    /// The shared base state.
    pub base: Arc<SharedWorldState<F>>,
    /// State for the *root & entry* of compilation.
    /// The world forbids direct access to files outside this directory.
    pub task: Arc<LazyHash<TaskState>>,
    /// Provides path-based data access for typst compiler.
    pub vfs: Vfs<F::AccessModel>,
}

impl<F: CompilerFeat> Deref for WorldState<F> {
    type Target = Arc<SharedWorldState<F>>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<F: CompilerFeat> Drop for WorldState<F> {
    fn drop(&mut self) {
        self.base.gc(&self.revision);
    }
}

impl<F: CompilerFeat> WorldState<F> {
    pub fn entry(&self) -> &EntryState {
        &self.task.entry
    }

    pub fn inputs(&self) -> &Arc<LazyHash<Dict>> {
        &self.task.inputs
    }

    pub fn revision(&self) -> NonZeroUsize {
        *self.revision.value
    }

    pub fn snapshot(&self) -> CompilerWorld<F> {
        self.snapshot_with(TaskInputs::default())
    }

    pub fn snapshot_with(&self, mutant: TaskInputs) -> CompilerWorld<F> {
        let (rev, task) = {
            let _lg = self.rev_lock.read();
            let rev = self.revision.acquire();
            let mut task = self.task.clone();
            if let Some(entry) = mutant.entry {
                Arc::make_mut(&mut task).entry = entry;
            }
            if let Some(inputs) = mutant.inputs {
                Arc::make_mut(&mut task).inputs = inputs;
            }
            (rev, task)
        };
        self.base.create_world(rev, task, self.vfs.snapshot())
    }

    /// Resolve the real path for a file id.
    pub fn path_for_id(&self, id: TypstFileId) -> Result<PathBuf, FileError> {
        if id == *DETACHED_ENTRY {
            return Ok(DETACHED_ENTRY.vpath().as_rooted_path().to_owned());
        }

        // Determine the root path relative to which the file path
        // will be resolved.
        let root = match id.package() {
            Some(spec) => self.registry.resolve(spec)?,
            None => self
                .entry()
                .root()
                .ok_or(FileError::Other(Some(eco_format!(
                    "cannot access directory without root: state: {:?}",
                    self.entry()
                ))))?,
        };

        // Join the path to the root. If it tries to escape, deny
        // access. Note: It can still escape via symlinks.
        id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
    }
}

type FileQuery<T> = QueryRef<T, FileError>;

#[derive(Default)]
pub struct ParseState {
    last_gc_revision: Mutex<usize>,
    /// The cache entries for each paths
    cache_entries: FxDashMap<TypstFileId, ParseCache>,
}

impl fmt::Debug for ParseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParseState").finish()
    }
}

impl ParseState {
    pub fn entry(&self, rev: NonZeroUsize, id: TypstFileId, content: Bytes) -> Arc<ParseSlot> {
        let mut entry = self.cache_entries.entry(id).or_default();
        let base = entry.prev.clone();

        let slot = entry.computes.entry(content.clone()).or_insert_with(|| {
            Arc::new(ParseSlot {
                last_accessed_rev: rev,
                base,
                replace: content,
                id,
                latest_success: std::sync::OnceLock::new(),
                compute: std::sync::OnceLock::new(),
            })
        });
        let slot = slot.clone();

        if slot.last_accessed_rev.get() > entry.last_accessed_rev {
            entry.last_accessed_rev = slot.last_accessed_rev.get();
            entry.prev = Some(slot.clone());
        }

        slot
    }

    pub(crate) fn gc(&self, revision: NonZeroUsize) {
        // todo: gc correctly
        let mut self_ = self.last_gc_revision.lock();
        if *self_ >= revision.get() {
            return;
        }
        *self_ = revision.get();
        let rev = *self_;
        self.cache_entries
            .retain(|_, r| r.last_accessed_rev + 15 >= rev)
    }
}

#[derive(Default)]
pub struct ParseCache {
    last_accessed_rev: usize,
    prev: Option<Arc<ParseSlot>>,
    computes: FxHashMap<Bytes, Arc<ParseSlot>>,
}

pub struct ParseSlot {
    last_accessed_rev: NonZeroUsize,
    base: Option<Arc<ParseSlot>>,
    id: TypstFileId,
    replace: Bytes,
    latest_success: std::sync::OnceLock<Source>,
    compute: std::sync::OnceLock<FileResult<Source>>,
}

impl ParseSlot {
    fn latest_success(&self) -> Option<&Source> {
        self.latest_success
            .get()
            .or_else(|| self.base.as_ref()?.latest_success())
    }

    pub fn parse(&self) -> &FileResult<Source> {
        self.compute.get_or_init(|| {
            let next = from_utf8_or_bom(&self.replace)?.to_owned();

            // otherwise reparse the source
            let new = self.latest_success.get_or_init(|| {
                let prev = self.base.as_ref().and_then(|e| e.latest_success());
                match prev.cloned() {
                    Some(mut source) => {
                        source.replace(&next);
                        source.clone()
                    }
                    // Return a new source if we don't have a reparse feature or no prev
                    _ => Source::new(self.id, next),
                }
            });

            Ok(new.clone())
        })
    }
}

pub struct SourceCache {
    pub fid: FileId,
    pub source: FileQuery<Source>,
    pub buffer: FileQuery<Bytes>,
}

pub struct SourceState {
    pub revision: NonZeroUsize,
    pub slots: Arc<Mutex<FxHashMap<TypstFileId, SourceCache>>>,
}

impl SourceState {}

/// Convert a byte slice to a string, removing UTF-8 BOM if present.
fn from_utf8_or_bom(buf: &[u8]) -> FileResult<&str> {
    Ok(std::str::from_utf8(if buf.starts_with(b"\xef\xbb\xbf") {
        // remove UTF-8 BOM
        &buf[3..]
    } else {
        // Assume UTF-8
        buf
    })?)
}

#[comemo::memoize]
fn create_library(inputs: Arc<LazyHash<Dict>>) -> Arc<LazyHash<Library>> {
    let lib = typst::Library::builder()
        .with_inputs(inputs.deref().deref().clone())
        .build();

    Arc::new(LazyHash::new(lib))
}
