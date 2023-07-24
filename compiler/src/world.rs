use std::{
    cell::Cell,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use chrono::Datelike;
use comemo::Prehashed;
use serde::{Deserialize, Serialize};
use typst::{
    diag::FileResult,
    eval::{Datetime, Library},
    font::{Font, FontBook},
    syntax::Source,
    World,
};

use typst_ts_core::{
    artifact_ir::ArtifactHeader,
    font::{FontProfile, FontResolverImpl},
    package::PackageError,
    Bytes, FontResolver, TypstFileId as FileId,
};

use crate::{
    package::Registry as PackageRegistry,
    vfs::{AccessModel as VfsAccessModel, Vfs},
    workspace::dependency::{DependencyTree, DependentFileInfo},
};

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

/// type trait interface of [`CompilerWorld`].
pub trait CompilerFeat {
    /// Specify the access model for VFS.
    type AccessModel: VfsAccessModel + Sized;
    /// Specify the package registry.
    type Registry: PackageRegistry + Sized;
}

/// A world that provides access to the operating system.
pub struct CompilerWorld<F: CompilerFeat> {
    /// Path to the root directory of compilation.
    /// The world forbids direct access to files outside this directory.
    pub root: Arc<Path>,
    /// Identifier of the main file.
    /// After resetting the world, this is set to a detached file.
    pub main: FileId,

    /// Provides library for typst compiler.
    library: Prehashed<Library>,
    /// Provides font management for typst compiler.
    pub font_resolver: FontResolverImpl,
    /// Provides package management for typst compiler.
    pub registry: F::Registry,
    /// Provides path-based data access for typst compiler.
    vfs: Vfs<F::AccessModel>,

    /// The date of today, which is fetched once per compilation.
    today: Cell<Option<Datetime>>,
}

impl<F: CompilerFeat> CompilerWorld<F> {
    /// Create a [`CompilerWorld`] with feature implementation.
    ///
    /// Although this function is public, it is always unstable and not intended
    /// to be used directly.
    /// + See [`crate::TypstSystemWorld::new`] for system environment.
    /// + See [`crate::TypstBrowserWorld::new`] for browser environment.
    pub fn new_raw(
        root_dir: PathBuf,
        vfs: Vfs<F::AccessModel>,
        registry: F::Registry,
        font_resolver: FontResolverImpl,
    ) -> Self {
        // Hook up the lang items.
        // todo: bad upstream changes
        let library = Prehashed::new(typst_library::build());
        typst::eval::set_lang_items(library.items.clone());

        Self {
            root: root_dir.into(),
            main: FileId::detached(),

            library,
            font_resolver,
            registry,
            vfs,

            today: Cell::new(None),
        }
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    /// The standard library.
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    /// Access the main source file.
    fn main(&self) -> Source {
        self.source(self.main).unwrap()
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
        self.vfs.resolve(&self.path_for_id(id)?, id)
    }

    /// Try to access the specified file.
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.vfs.file(&self.path_for_id(id).unwrap())
    }

    /// Get the current date.
    ///
    /// If no offset is specified, the local date should be chosen. Otherwise,
    /// the UTC date should be chosen with the corresponding offset in hours.
    ///
    /// If this function returns `None`, Typst's `datetime` function will
    /// return an error.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        if self.today.get().is_none() {
            let datetime = match offset {
                None => chrono::Local::now().naive_local(),
                Some(o) => (chrono::Utc::now() + chrono::Duration::hours(o)).naive_utc(),
            };

            self.today.set(Some(Datetime::from_ymd(
                datetime.year(),
                datetime.month().try_into().ok()?,
                datetime.day().try_into().ok()?,
            )?))
        }

        self.today.get()
    }
}

impl<F: CompilerFeat> CompilerWorld<F> {
    /// Reset the world for a new lifecycle (of garbage collection).
    pub fn reset(&mut self) {
        self.vfs.reset();

        self.today.set(None);
    }

    pub fn reset_shadow(&mut self) {
        self.vfs.reset_shadow()
    }

    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.vfs.do_reparse = do_reparse;
    }

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path, source_id: FileId) -> FileResult<()> {
        self.vfs.resolve(path, source_id).map(|_| ())
    }

    /// Override the content of a file.
    pub fn resolve_with<P: AsRef<Path>>(
        &self,
        path: P,
        source_id: FileId,
        content: &str,
    ) -> FileResult<()> {
        self.vfs.resolve_with(path, source_id, content).map(|_| ())
    }

    pub fn remove_shadow(&self, path: &Path) {
        self.vfs.remove_shadow(path)
    }

    /// Resolve the real path for a file id.
    pub fn path_for_id(&self, id: FileId) -> Result<PathBuf, PackageError> {
        // Determine the root path relative to which the file path
        // will be resolved.
        let root = match id.package() {
            Some(spec) => self.registry.resolve(spec)?,
            None => self.root.clone(),
        };

        Ok(root.join(id.path().strip_prefix(Path::new("/")).unwrap()))
    }

    /// Get found dependencies in current state of vfs.
    pub fn get_dependencies(&self) -> DependencyTree {
        let vfs_dependencies =
            self.vfs
                .iter_dependencies()
                .map(|(path, mtime)| DependentFileInfo {
                    path: path.to_owned(),
                    mtime: mtime
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_micros() as u64,
                });

        DependencyTree::from_iter(&self.root, vfs_dependencies)
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

impl<'a, F: CompilerFeat> codespan_reporting::files::Files<'a> for CompilerWorld<F> {
    /// A unique identifier for files in the file provider. This will be used
    /// for rendering `diagnostic::Label`s in the corresponding source files.
    type FileId = FileId;

    /// The user-facing name of a file, to be displayed in diagnostics.
    type Name = std::path::Display<'a>;

    /// The source code of a file.
    type Source = Source;

    /// The user-facing name of a file.
    fn name(&'a self, id: FileId) -> CodespanResult<Self::Name> {
        Ok(id.path().display())
    }

    /// The source code of a file.
    fn source(&'a self, id: FileId) -> CodespanResult<Self::Source> {
        World::source(self, id).map_err(|_e| CodespanError::FileMissing)
    }

    /// See [`codespan_reporting::files::Files::line_index`].
    fn line_index(&'a self, id: FileId, given: usize) -> CodespanResult<usize> {
        self.map_source_or_default(id, 0, |source| {
            source
                .byte_to_line(given)
                .ok_or_else(|| CodespanError::IndexTooLarge {
                    given,
                    max: source.len_bytes(),
                })
        })
    }

    /// See [`codespan_reporting::files::Files::column_number`].
    fn column_number(&'a self, id: FileId, _: usize, given: usize) -> CodespanResult<usize> {
        self.map_source_or_default(id, 0, |source| {
            source.byte_to_column(given).ok_or_else(|| {
                let max = source.len_bytes();
                if given <= max {
                    CodespanError::InvalidCharBoundary { given }
                } else {
                    CodespanError::IndexTooLarge { given, max }
                }
            })
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
    pub artifact_header: ArtifactHeader,
    pub artifact_data: String,
}
