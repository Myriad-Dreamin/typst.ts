use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Datelike, Local};
use comemo::Prehashed;
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use typst::{
    diag::{eco_format, At, EcoString, FileError, FileResult, SourceResult},
    foundations::{Datetime, Dict},
    syntax::{Source, Span, VirtualPath},
    text::{Font, FontBook},
    Library, World,
};

use reflexo_world::{SharedState, SourceCache, SourceDb};
use typst_ts_core::{
    config::compiler::{EntryState, DETACHED_ENTRY},
    package::PackageSpec,
    Bytes, FontResolver, ImmutPath, TypstFileId as FileId,
};

use crate::{
    package::Registry as PackageRegistry,
    parser::{
        get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
        SemanticTokensLegend,
    },
    service::{EntryManager, EntryReader},
    vfs::{notify::FilesystemEvent, AccessModel as VfsAccessModel, Vfs},
    ShadowApi, WorldDeps,
};

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

/// type trait interface of [`CompilerWorld`].
pub trait CompilerFeat {
    /// Specify the font resolver for typst compiler.
    type FontResolver: FontResolver + Send + Sync + Sized;
    /// Specify the access model for VFS.
    type AccessModel: VfsAccessModel + Send + Sync + Sized;
    /// Specify the package registry.
    type Registry: PackageRegistry + Send + Sync + Sized;
}

/// A world that provides access to the operating system.
#[derive(Debug)]
pub struct CompilerUniverse<F: CompilerFeat> {
    /// State for the *root & entry* of compilation.
    /// The world forbids direct access to files outside this directory.
    pub entry: EntryState,
    /// Additional input arguments to compile the entry file.
    pub inputs: Arc<Prehashed<Dict>>,
    /// Whether to reparse the source files.
    pub do_reparse: bool,

    /// Provides font management for typst compiler.
    pub font_resolver: Arc<F::FontResolver>,
    /// Provides package management for typst compiler.
    pub registry: Arc<F::Registry>,
    /// Provides path-based data access for typst compiler.
    pub vfs: Arc<RwLock<Vfs<F::AccessModel>>>,

    /// The current revision of the source database.
    pub revision: Mutex<usize>,
    /// Shared state for source cache.
    pub shared: Arc<RwLock<SharedState<SourceCache>>>,
}

impl<F: CompilerFeat> CompilerUniverse<F> {
    /// Create a [`CompilerUniverse`] with feature implementation.
    ///
    /// Although this function is public, it is always unstable and not intended
    /// to be used directly.
    /// + See [`crate::TypstSystemUniverse::new`] for system environment.
    /// + See [`crate::TypstBrowserUniverse::new`] for browser environment.
    pub fn new_raw(
        entry: EntryState,
        inputs: Option<Arc<Prehashed<Dict>>>,
        vfs: Arc<RwLock<Vfs<F::AccessModel>>>,
        registry: F::Registry,
        font_resolver: Arc<F::FontResolver>,
    ) -> Self {
        Self {
            entry,
            inputs: inputs.unwrap_or_default(),
            do_reparse: true,

            revision: Mutex::new(0),
            shared: Arc::new(RwLock::new(SharedState::default())),

            // library: create_library(inputs.unwrap_or_default()),
            font_resolver,
            registry: Arc::new(registry),
            vfs,
        }
    }

    pub fn set_inputs(&mut self, inputs: Arc<Prehashed<Dict>>) {
        self.inputs = inputs;
    }

    /// Wrap driver with a given entry file.
    pub fn with_entry_file(mut self, entry_file: PathBuf) -> Self {
        self.set_entry_file(entry_file.as_path().into()).unwrap();
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

        self.mutate_entry(state).map(|_| ())?;
        Ok(())
    }

    #[inline]
    pub fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.vfs.write().notify_fs_event(event)
    }

    pub fn spawn(&self) -> CompilerWorld<F> {
        let mut rev_lock = self.revision.lock();
        *rev_lock += 1;

        CompilerWorld {
            entry: self.entry.clone(),
            library: create_library(self.inputs.clone()),
            font_resolver: self.font_resolver.clone(),
            registry: self.registry.clone(),
            vfs: self.vfs.clone(),
            source_db: SourceDb {
                revision: *rev_lock,
                do_reparse: self.do_reparse,
                shared: self.shared.clone(),
                slots: Default::default(),
            },
            now: OnceCell::new(),
        }
    }
}

impl<F: CompilerFeat> CompilerUniverse<F> {
    /// Reset the world for a new lifecycle (of garbage collection).
    pub fn reset(&mut self) {
        self.vfs.write().reset();
        // todo: shared state
    }

    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.do_reparse = do_reparse;
    }

    /// Resolve the real path for a file id.
    pub fn path_for_id(&self, id: FileId) -> Result<PathBuf, FileError> {
        if id == *DETACHED_ENTRY {
            return Ok(DETACHED_ENTRY.vpath().as_rooted_path().to_owned());
        }

        // Determine the root path relative to which the file path
        // will be resolved.
        let root = match id.package() {
            Some(spec) => self.registry.resolve(spec)?,
            None => self.entry.root().ok_or(FileError::Other(Some(eco_format!(
                "cannot access directory without root: state: {:?}",
                self.entry
            ))))?,
        };

        // Join the path to the root. If it tries to escape, deny
        // access. Note: It can still escape via symlinks.
        id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
    }

    pub fn get_semantic_token_legend(&self) -> Arc<SemanticTokensLegend> {
        Arc::new(get_semantic_tokens_legend())
    }

    pub fn get_semantic_tokens(
        &self,
        file_path: Option<String>,
        encoding: OffsetEncoding,
    ) -> Arc<Vec<SemanticToken>> {
        let world = self.spawn();
        let src = &file_path
            .and_then(|e| {
                let relative_path = Path::new(&e).strip_prefix(&self.workspace_root()?).ok()?;

                let source_id = FileId::new(None, VirtualPath::new(relative_path));
                world.source(source_id).ok()
            })
            .unwrap_or_else(|| world.main());

        Arc::new(get_semantic_tokens_full(src, encoding))
    }
}

impl<F: CompilerFeat> ShadowApi for CompilerUniverse<F> {
    #[inline]
    fn _shadow_map_id(&self, file_id: FileId) -> FileResult<PathBuf> {
        self.path_for_id(file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.vfs.read().shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.vfs.write().reset_shadow()
    }

    #[inline]
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.vfs.read().map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&self, path: &Path) -> FileResult<()> {
        self.vfs.read().remove_shadow(path);

        Ok(())
    }
}

impl<F: CompilerFeat> EntryReader for CompilerUniverse<F> {
    fn entry_state(&self) -> EntryState {
        self.entry.clone()
    }
}

impl<F: CompilerFeat> EntryManager for CompilerUniverse<F> {
    fn reset(&mut self) -> SourceResult<()> {
        self.reset();
        Ok(())
    }

    fn mutate_entry(&mut self, mut state: EntryState) -> SourceResult<EntryState> {
        self.reset();
        std::mem::swap(&mut self.entry, &mut state);
        Ok(state)
    }
}

pub struct CompilerWorld<F: CompilerFeat> {
    /// State for the *root & entry* of compilation.
    /// The world forbids direct access to files outside this directory.
    pub entry: EntryState,

    /// Provides library for typst compiler.
    pub library: Arc<Prehashed<Library>>,
    /// Provides font management for typst compiler.
    pub font_resolver: Arc<F::FontResolver>,
    /// Provides package management for typst compiler.
    pub registry: Arc<F::Registry>,
    /// Provides path-based data access for typst compiler.
    pub vfs: Arc<RwLock<Vfs<F::AccessModel>>>,

    /// Provides source database for typst compiler.
    pub source_db: SourceDb,
    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceCell<DateTime<Local>>,
}

impl<F: CompilerFeat> Drop for CompilerWorld<F> {
    fn drop(&mut self) {
        let state = self.source_db.shared.clone();
        let mut state = state.write();
        let source_db = std::mem::take(&mut self.source_db);
        source_db.commit_impl(&mut state);
    }
}

impl<F: CompilerFeat> CompilerWorld<F> {
    pub fn task(&self, inputs: Option<Arc<Prehashed<Dict>>>) -> CompilerWorld<F> {
        // Fetch to avoid inconsistent state.
        let _ = self.today(None);

        CompilerWorld {
            library: inputs
                .map(create_library)
                .unwrap_or_else(|| self.library.clone()),
            entry: self.entry.clone(),
            font_resolver: self.font_resolver.clone(),
            registry: self.registry.clone(),
            vfs: self.vfs.clone(),
            source_db: self.source_db.clone(),
            now: self.now.clone(),
        }
    }

    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.source_db.do_reparse = do_reparse;
    }

    /// Resolve the real path for a file id.
    pub fn path_for_id(&self, id: FileId) -> Result<PathBuf, FileError> {
        if id == *DETACHED_ENTRY {
            return Ok(DETACHED_ENTRY.vpath().as_rooted_path().to_owned());
        }

        // Determine the root path relative to which the file path
        // will be resolved.
        let root = match id.package() {
            Some(spec) => self.registry.resolve(spec)?,
            None => self.entry.root().ok_or(FileError::Other(Some(eco_format!(
                "cannot access directory without root: state: {:?}",
                self.entry
            ))))?,
        };

        // Join the path to the root. If it tries to escape, deny
        // access. Note: It can still escape via symlinks.
        id.vpath().resolve(&root).ok_or(FileError::AccessDenied)
    }
    /// Lookup a source file by id.
    #[track_caller]
    fn lookup(&self, id: FileId) -> Source {
        self.source(id)
            .expect("file id does not point to any source file")
    }

    fn map_source_or_default<T>(
        &self,
        id: FileId,
        default_v: T,
        f: impl FnOnce(Source) -> CodespanResult<T>,
    ) -> CodespanResult<T> {
        match World::source(self, id).ok() {
            Some(source) => f(source),
            None => Ok(default_v),
        }
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    /// The standard library.
    fn library(&self) -> &Prehashed<Library> {
        self.library.as_ref()
    }

    /// Access the main source file.
    fn main(&self) -> Source {
        self.source(self.entry.main().unwrap_or_else(|| *DETACHED_ENTRY))
            .unwrap()
    }

    /// Metadata about all known fonts.
    fn font(&self, id: usize) -> Option<Font> {
        self.font_resolver.font(id)
    }

    /// Try to access the specified file.
    fn book(&self) -> &Prehashed<FontBook> {
        self.font_resolver.font_book()
    }

    /// Try to access the specified source file.
    ///
    /// The returned `Source` file's [id](Source::id) does not have to match the
    /// given `id`. Due to symlinks, two different file id's can point to the
    /// same on-disk file. Implementors can deduplicate and return the same
    /// `Source` if they want to, but do not have to.
    fn source(&self, id: FileId) -> FileResult<Source> {
        static DETACH_SOURCE: once_cell::sync::Lazy<Source> =
            once_cell::sync::Lazy::new(|| Source::new(*DETACHED_ENTRY, String::new()));

        if id == *DETACHED_ENTRY {
            return Ok(DETACH_SOURCE.clone());
        }

        let vfs = self.vfs.read();
        let fid = vfs.file_id(&self.path_for_id(id)?);
        self.source_db.source(id, fid, vfs.deref())
    }

    /// Try to access the specified file.
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let vfs = self.vfs.read();
        let fid = vfs.file_id(&self.path_for_id(id)?);
        self.source_db.file(id, fid, vfs.deref())
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

impl<F: CompilerFeat> WorldDeps for CompilerWorld<F> {
    #[inline]
    fn iter_dependencies(&self, f: &mut dyn FnMut(ImmutPath)) {
        self.source_db
            .iter_dependencies_dyn(self.vfs.read().deref(), f)
    }
}

impl<F: CompilerFeat> EntryReader for CompilerWorld<F> {
    fn entry_state(&self) -> EntryState {
        self.entry.clone()
    }
}

impl<'a, F: CompilerFeat> codespan_reporting::files::Files<'a> for CompilerWorld<F> {
    /// A unique identifier for files in the file provider. This will be used
    /// for rendering `diagnostic::Label`s in the corresponding source files.
    type FileId = FileId;

    /// The user-facing name of a file, to be displayed in diagnostics.
    type Name = String;

    /// The source code of a file.
    type Source = Source;

    /// The user-facing name of a file.
    fn name(&'a self, id: FileId) -> CodespanResult<Self::Name> {
        let vpath = id.vpath();
        Ok(if let Some(package) = id.package() {
            format!("{package}{}", vpath.as_rooted_path().display())
        } else {
            match self.entry.root() {
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
    fn source(&'a self, id: FileId) -> CodespanResult<Self::Source> {
        Ok(self.lookup(id))
    }

    /// See [`codespan_reporting::files::Files::line_index`].
    fn line_index(&'a self, id: FileId, given: usize) -> CodespanResult<usize> {
        let source = self.lookup(id);
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    /// See [`codespan_reporting::files::Files::column_number`].
    fn column_number(&'a self, id: FileId, _: usize, given: usize) -> CodespanResult<usize> {
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
    fn line_range(&'a self, id: FileId, given: usize) -> CodespanResult<std::ops::Range<usize>> {
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

#[comemo::memoize]
fn create_library(inputs: Arc<Prehashed<Dict>>) -> Arc<Prehashed<Library>> {
    let lib = typst::Library::builder()
        .with_inputs(inputs.deref().deref().clone())
        .build();

    Arc::new(Prehashed::new(lib))
}
