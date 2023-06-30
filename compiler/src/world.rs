use std::{
    cell::Cell,
    path::{Path, PathBuf},
    time::SystemTime,
};

use chrono::Datelike;
use comemo::Prehashed;
use serde::{Deserialize, Serialize};
use typst::{
    diag::{FileResult, PackageError, PackageResult},
    eval::{Datetime, Library},
    file::{FileId, PackageSpec},
    font::{Font, FontBook},
    syntax::Source,
    World,
};

use typst_ts_core::{
    artifact_ir::ArtifactHeader,
    font::{FontProfile, FontResolverImpl},
    Bytes, FontResolver,
};

use crate::{
    vfs::{AccessModel, Vfs},
    workspace::dependency::{DependencyTree, DependentFileInfo},
};

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

pub trait CompilerFeat {
    type M: AccessModel + Sized;
}

/// A world that provides access to the operating system.
pub struct CompilerWorld<F: CompilerFeat> {
    pub root: PathBuf,
    pub main: FileId,

    library: Prehashed<Library>,
    pub font_resolver: FontResolverImpl,
    vfs: Vfs<F::M>,
    today: Cell<Option<Datetime>>,
}

impl<F: CompilerFeat> CompilerWorld<F> {
    pub fn new_raw(root_dir: PathBuf, vfs: Vfs<F::M>, font_resolver: FontResolverImpl) -> Self {
        // Hook up the lang items.
        // todo: bad upstream changes
        let library = Prehashed::new(typst_library::build());
        typst::eval::set_lang_items(library.items.clone());

        Self {
            root: root_dir,
            library,
            font_resolver,
            main: FileId::detached(),
            today: Cell::new(None),
            vfs,
        }
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> Source {
        self.source(self.main).unwrap()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.vfs.source(id)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.font_resolver.font_book()
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.font_resolver.font(id)
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.vfs.file(&self.path_for_id(id).unwrap())
    }

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
    /// Set the `do_reparse` flag.
    pub fn set_do_reparse(&mut self, do_reparse: bool) {
        self.vfs.do_reparse = do_reparse;
    }

    /// Get source id by path with filesystem content.
    pub fn resolve(&self, path: &Path, source_id: FileId) -> FileResult<()> {
        self.vfs.resolve(path, source_id)
    }

    pub fn resolve_with<P: AsRef<Path>>(
        &self,
        path: P,
        source_id: FileId,
        content: &str,
    ) -> FileResult<()> {
        self.vfs.resolve_with(path, source_id, content)
    }

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

    pub fn reset(&mut self) {
        self.vfs.reset();

        self.today.set(None);
    }

    fn path_for_id(&self, id: FileId) -> PackageResult<PathBuf> {
        // Determine the root path relative to which the file path
        // will be resolved.
        let root = match id.package() {
            Some(spec) => prepare_package(spec)?,
            None => self.root.clone(),
        };

        Ok(root.join(id.path()))
    }
}

impl<'a, F: CompilerFeat> codespan_reporting::files::Files<'a> for CompilerWorld<F> {
    type FileId = FileId;
    type Name = std::path::Display<'a>;
    type Source = Source;

    fn name(&'a self, id: FileId) -> CodespanResult<Self::Name> {
        Ok(id.path().display())
    }

    fn source(&'a self, id: FileId) -> CodespanResult<Self::Source> {
        World::source(self, id).map_err(|_e| CodespanError::FileMissing)
    }

    fn line_index(&'a self, id: FileId, given: usize) -> CodespanResult<usize> {
        let source = World::source(self, id).ok();
        source
            .map(|source| {
                source
                    .byte_to_line(given)
                    .ok_or_else(|| CodespanError::IndexTooLarge {
                        given,
                        max: source.len_bytes(),
                    })
            })
            .unwrap_or(Ok(0))
    }

    fn line_range(&'a self, id: FileId, given: usize) -> CodespanResult<std::ops::Range<usize>> {
        let source = World::source(self, id).ok();
        source
            .map(|source| {
                source
                    .line_to_range(given)
                    .ok_or_else(|| CodespanError::LineTooLarge {
                        given,
                        max: source.len_lines(),
                    })
            })
            .unwrap_or(Ok(0..0))
    }

    fn column_number(&'a self, id: FileId, _: usize, given: usize) -> CodespanResult<usize> {
        let source = World::source(self, id).ok();
        source
            .map(|source| {
                source.byte_to_column(given).ok_or_else(|| {
                    let max = source.len_bytes();
                    if given <= max {
                        CodespanError::InvalidCharBoundary { given }
                    } else {
                        CodespanError::IndexTooLarge { given, max }
                    }
                })
            })
            .unwrap_or(Ok(0))
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

/// Make a package available in the on-disk cache.
fn prepare_package(spec: &PackageSpec) -> PackageResult<PathBuf> {
    let subdir = format!(
        "typst/packages/{}/{}-{}",
        spec.namespace, spec.name, spec.version
    );

    if let Some(data_dir) = dirs::data_dir() {
        let dir = data_dir.join(&subdir);
        if dir.exists() {
            return Ok(dir);
        }
    }

    if let Some(cache_dir) = dirs::cache_dir() {
        let dir = cache_dir.join(&subdir);

        // Download from network if it doesn't exist yet.
        if spec.namespace == "preview" && !dir.exists() {
            download_package(spec, &dir)?;
        }

        if dir.exists() {
            return Ok(dir);
        }
    }

    Err(PackageError::NotFound(spec.clone()))
}

/// Download a package over the network.
fn download_package(spec: &PackageSpec, package_dir: &Path) -> PackageResult<()> {
    // The `@preview` namespace is the only namespace that supports on-demand
    // fetching.
    assert_eq!(spec.namespace, "preview");

    let url = format!(
        "https://packages.typst.org/preview/{}-{}.tar.gz",
        spec.name, spec.version
    );

    print_downloading(spec).unwrap();
    let reader = match ureq::get(&url).call() {
        Ok(response) => response.into_reader(),
        Err(ureq::Error::Status(404, _)) => return Err(PackageError::NotFound(spec.clone())),
        Err(_) => return Err(PackageError::NetworkFailed),
    };

    let decompressed = flate2::read::GzDecoder::new(reader);
    tar::Archive::new(decompressed)
        .unpack(package_dir)
        .map_err(|_| {
            std::fs::remove_dir_all(package_dir).ok();
            PackageError::MalformedArchive
        })
}

/// Print that a package downloading is happening.
fn print_downloading(_spec: &PackageSpec) -> std::io::Result<()> {
    // let mut w = color_stream();
    // let styles = term::Styles::default();

    // w.set_color(&styles.header_help)?;
    // write!(w, "downloading")?;

    // w.reset()?;
    // writeln!(w, " {spec}")
    Ok(())
}
