use std::{
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use chrono::{DateTime, Datelike, Local};
use comemo::Prehashed;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use typst::{
    diag::{eco_format, At, EcoString, FileError, FileResult, Hint, SourceResult},
    foundations::{Datetime, Dict},
    syntax::{Source, Span, VirtualPath},
    text::{Font, FontBook},
    Library, World,
};

use typst_ts_core::{
    config::compiler::{EntryState, DETACHED_ENTRY},
    font::FontProfile,
    package::PackageSpec,
    Bytes, FontResolver, ImmutPath, TypstFileId as FileId,
};

use crate::{
    dependency::{DependencyTree, DependentFileInfo},
    package::Registry as PackageRegistry,
    parser::{
        get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
        SemanticTokensLegend,
    },
    service::{CompileEnv, EntryManager, EnvWorld},
    vfs::{notify::FilesystemEvent, AccessModel as VfsAccessModel, Vfs},
    NotifyApi, ShadowApi, Time,
};

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

/// type trait interface of [`CompilerWorld`].
pub trait CompilerFeat {
    /// Specify the font resolver for typst compiler.
    type FontResolver: FontResolver + Sized;
    /// Specify the access model for VFS.
    type AccessModel: VfsAccessModel + Sized;
    /// Specify the package registry.
    type Registry: PackageRegistry + Sized;
}

/// A world that provides access to the operating system.
#[derive(Debug)]
pub struct CompilerWorld<F: CompilerFeat> {
    /// State for the *root & entry* of compilation.
    /// The world forbids direct access to files outside this directory.
    pub entry: EntryState,
    /// Additional input arguments to compile the entry file.
    pub inputs: Arc<Prehashed<Dict>>,

    /// Provides library for typst compiler.
    pub library: Option<Arc<Prehashed<Library>>>,
    /// Provides font management for typst compiler.
    pub font_resolver: F::FontResolver,
    /// Provides package management for typst compiler.
    pub registry: F::Registry,
    /// Provides path-based data access for typst compiler.
    pub vfs: Vfs<F::AccessModel>,

    /// The current datetime if requested. This is stored here to ensure it is
    /// always the same within one compilation. Reset between compilations.
    now: OnceCell<DateTime<Local>>,
}

impl<F: CompilerFeat> CompilerWorld<F> {
    /// Create a [`CompilerWorld`] with feature implementation.
    ///
    /// Although this function is public, it is always unstable and not intended
    /// to be used directly.
    /// + See [`crate::TypstSystemWorld::new`] for system environment.
    /// + See [`crate::TypstBrowserWorld::new`] for browser environment.
    pub fn new_raw(
        entry: EntryState,
        vfs: Vfs<F::AccessModel>,
        registry: F::Registry,
        font_resolver: F::FontResolver,
    ) -> Self {
        Self {
            entry,
            inputs: Arc::new(Prehashed::new(Dict::new())),

            library: None,
            font_resolver,
            registry,
            vfs,

            now: OnceCell::new(),
        }
    }

    pub fn set_inputs(&mut self, inputs: Arc<Prehashed<Dict>>) {
        self.inputs = inputs;
    }
}

#[comemo::memoize]
fn create_library(inputs: Arc<Prehashed<Dict>>) -> Arc<Prehashed<Library>> {
    let lib = typst::Library::builder()
        .with_inputs(inputs.deref().deref().clone())
        .build();

    Arc::new(Prehashed::new(lib))
}

impl<F: CompilerFeat> EnvWorld for CompilerWorld<F> {
    fn ensure_env(&mut self) -> SourceResult<()> {
        if self.library.is_none() {
            return Err(eco_format!("library not initialized"))
                .hint(eco_format!("do you forget to run compilation?"))
                .at(Span::detached());
        }

        Ok(())
    }

    fn prepare_env(&mut self, _env: &mut CompileEnv) -> SourceResult<()> {
        // Hook up the lang items.
        // todo: bad upstream changes
        self.library = Some(create_library(self.inputs.clone()));

        Ok(())
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    /// The standard library.
    fn library(&self) -> &Prehashed<Library> {
        self.library.as_ref().unwrap()
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

        self.vfs.resolve(&self.path_for_id(id)?, id)
    }

    /// Try to access the specified file.
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.vfs.file(&self.path_for_id(id)?)
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

impl<F: CompilerFeat> CompilerWorld<F> {
    /// Reset the world for a new lifecycle (of garbage collection).
    pub fn reset(&mut self) {
        self.vfs.reset();

        self.now.take();
    }

    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.vfs.do_reparse = do_reparse;
    }

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path, source_id: FileId) -> FileResult<()> {
        self.vfs.resolve(path, source_id).map(|_| ())
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

    /// Get found dependencies in current state of vfs.
    pub fn get_dependencies(&self) -> Option<DependencyTree> {
        let root = self.entry.root()?;

        let deps = self.vfs.iter_dependencies();
        let vfs_dependencies = deps
            .filter_map(|(x, maybe_time)| Some((x, maybe_time.ok()?)))
            .map(|(path, mtime)| DependentFileInfo {
                path: path.as_ref().to_owned(),
                mtime: mtime.duration_since(Time::UNIX_EPOCH).unwrap().as_micros() as u64,
            });

        Some(DependencyTree::from_iter(&root, vfs_dependencies))
    }

    pub fn get_semantic_token_legend(&self) -> Arc<SemanticTokensLegend> {
        Arc::new(get_semantic_tokens_legend())
    }

    pub fn get_semantic_tokens(
        &self,
        file_path: Option<String>,
        encoding: OffsetEncoding,
    ) -> Arc<Vec<SemanticToken>> {
        let src = &file_path
            .and_then(|e| {
                let relative_path = Path::new(&e).strip_prefix(&self.workspace_root()?).ok()?;

                let source_id = FileId::new(None, VirtualPath::new(relative_path));
                self.source(source_id).ok()
            })
            .unwrap_or_else(|| self.main());

        Arc::new(get_semantic_tokens_full(src, encoding))
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

    /// Lookup a source file by id.
    #[track_caller]
    fn lookup(&self, id: FileId) -> Source {
        self.source(id)
            .expect("file id does not point to any source file")
    }
}

impl<F: CompilerFeat> ShadowApi for CompilerWorld<F> {
    #[inline]
    fn _shadow_map_id(&self, file_id: FileId) -> FileResult<PathBuf> {
        self.path_for_id(file_id)
    }

    #[inline]
    fn shadow_paths(&self) -> Vec<Arc<Path>> {
        self.vfs.shadow_paths()
    }

    #[inline]
    fn reset_shadow(&mut self) {
        self.vfs.reset_shadow()
    }

    #[inline]
    fn map_shadow(&self, path: &Path, content: Bytes) -> FileResult<()> {
        self.vfs.map_shadow(path, content)
    }

    #[inline]
    fn unmap_shadow(&self, path: &Path) -> FileResult<()> {
        self.vfs.remove_shadow(path);

        Ok(())
    }
}

impl<F: CompilerFeat> NotifyApi for CompilerWorld<F> {
    #[inline]
    fn iter_dependencies<'a>(&'a self, f: &mut dyn FnMut(&'a ImmutPath, FileResult<&Time>)) {
        self.vfs.iter_dependencies_dyn(f)
    }

    #[inline]
    fn notify_fs_event(&mut self, event: FilesystemEvent) {
        self.vfs.notify_fs_event(event)
    }
}

impl<F: CompilerFeat> EntryManager for CompilerWorld<F> {
    fn reset(&mut self) -> SourceResult<()> {
        self.reset();
        Ok(())
    }

    fn workspace_root(&self) -> Option<Arc<Path>> {
        self.entry.root().clone()
    }

    fn main_id(&self) -> Option<FileId> {
        self.entry.main()
    }

    fn entry_state(&self) -> EntryState {
        self.entry.clone()
    }

    fn mutate_entry(&mut self, mut state: EntryState) -> SourceResult<EntryState> {
        self.reset();
        std::mem::swap(&mut self.entry, &mut state);
        Ok(state)
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub font_profile: Option<FontProfile>,
    pub dependencies: DependencyTree,

    /// document specific data
    pub artifact_data: String,
}
