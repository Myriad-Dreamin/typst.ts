use std::{
    path::{Path, PathBuf},
    time::SystemTime,
};

use comemo::Prehashed;
use typst::{
    diag::FileResult,
    eval::Library,
    font::{Font, FontBook},
    syntax::{Source, SourceId},
    util::Buffer,
    World,
};
use typst_ts_core::{font::FontResolverImpl, FontResolver};

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
    root: PathBuf,
    pub main: SourceId,

    library: Prehashed<Library>,
    pub font_resolver: FontResolverImpl,
    vfs: Vfs<F::M>,
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
            main: SourceId::detached(),
            vfs,
        }
    }
}

impl<F: CompilerFeat> World for CompilerWorld<F> {
    fn root(&self) -> &Path {
        &self.root
    }

    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn main(&self) -> &Source {
        self.source(self.main)
    }

    fn resolve(&self, path: &Path) -> FileResult<SourceId> {
        self.vfs.resolve(path)
    }

    fn source(&self, id: SourceId) -> &Source {
        self.vfs.source(id)
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.font_resolver.font_book()
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.font_resolver.font(id)
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        self.vfs.file(path)
    }
}

impl<F: CompilerFeat> CompilerWorld<F> {
    pub fn resolve_with<P: AsRef<Path>>(&self, path: P, content: &str) -> FileResult<SourceId> {
        self.vfs.resolve_with(path, content)
    }

    pub fn dependant(&self, path: &Path) -> bool {
        self.vfs.dependant(path)
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
    }
}

impl<'a, F: CompilerFeat> codespan_reporting::files::Files<'a> for CompilerWorld<F> {
    type FileId = SourceId;
    type Name = std::path::Display<'a>;
    type Source = &'a str;

    fn name(&'a self, id: SourceId) -> CodespanResult<Self::Name> {
        Ok(World::source(self, id).path().display())
    }

    fn source(&'a self, id: SourceId) -> CodespanResult<Self::Source> {
        Ok(World::source(self, id).text())
    }

    fn line_index(&'a self, id: SourceId, given: usize) -> CodespanResult<usize> {
        let source = World::source(self, id);
        source
            .byte_to_line(given)
            .ok_or_else(|| CodespanError::IndexTooLarge {
                given,
                max: source.len_bytes(),
            })
    }

    fn line_range(&'a self, id: SourceId, given: usize) -> CodespanResult<std::ops::Range<usize>> {
        let source = World::source(self, id);
        source
            .line_to_range(given)
            .ok_or_else(|| CodespanError::LineTooLarge {
                given,
                max: source.len_lines(),
            })
    }

    fn column_number(&'a self, id: SourceId, _: usize, given: usize) -> CodespanResult<usize> {
        let source = World::source(self, id);
        source.byte_to_column(given).ok_or_else(|| {
            let max = source.len_bytes();
            if given <= max {
                CodespanError::InvalidCharBoundary { given }
            } else {
                CodespanError::IndexTooLarge { given, max }
            }
        })
    }
}
