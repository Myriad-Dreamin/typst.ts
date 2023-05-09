//! Derived from https://github.com/nvarner/typst-lsp/blob/f4b8bc7a967be3a720a1753b76a57f1528a99633/src/system_world.rs
//! Currently No Modification.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use append_only_vec::AppendOnlyVec;
use comemo::Prehashed;
use memmap2::Mmap;
use parking_lot::{MappedRwLockWriteGuard, RwLock, RwLockWriteGuard};
use typst::diag::{FileError, FileResult};
use typst::eval::Library;
use typst::font::{Font, FontBook, FontInfo};
use typst::syntax::{Source, SourceId};
use typst::util::{Buffer, PathExt};
use typst::World;
use typst_ts_core::config::CompileOpts;
use typst_ts_core::FontResolver;
use walkdir::WalkDir;

use crate::path::{PathHash, PathSlot};
use crate::system::LazyFile;
use crate::ReadFontLoader;
use typst_ts_core::{font::FontResolverImpl, FontSlot};

type CodespanResult<T> = Result<T, CodespanError>;
type CodespanError = codespan_reporting::files::Error;

/// A world that provides access to the operating system.
pub struct TypstSystemWorld {
    root: PathBuf,
    library: Prehashed<Library>,
    pub font_resolver: FontResolverImpl,
    hashes: RwLock<HashMap<PathBuf, FileResult<PathHash>>>,
    paths: RwLock<HashMap<PathHash, PathSlot>>,
    pub sources: AppendOnlyVec<Source>,
    pub main: SourceId,
}

impl TypstSystemWorld {
    pub fn new(opts: CompileOpts) -> Self {
        let mut searcher = SystemFontSearcher::new();
        searcher.search_system();
        searcher.add_embedded();
        for path in opts.font_paths {
            if path.is_dir() {
                searcher.search_dir(&path);
            } else {
                searcher.search_file(&path);
            }
        }

        // Hook up the lang items.
        // todo: bad upstream changes
        let library = Prehashed::new(typst_library::build());
        typst::eval::set_lang_items(library.items.clone());

        Self {
            root: opts.root_dir,
            library,
            font_resolver: FontResolverImpl::new(searcher.book, searcher.fonts),
            hashes: RwLock::default(),
            paths: RwLock::default(),
            sources: AppendOnlyVec::new(),
            main: SourceId::detached(),
        }
    }
}

impl World for TypstSystemWorld {
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
        self.slot(path)?
            .source
            .get_or_init(|| {
                let buf = read(path)?;
                let text = String::from_utf8(buf)?;
                Ok(self.insert(path, text))
            })
            .clone()
    }

    fn source(&self, id: SourceId) -> &Source {
        &self.sources[id.into_u16() as usize]
    }

    fn book(&self) -> &Prehashed<FontBook> {
        self.font_resolver.font_book()
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.font_resolver.font(id)
    }

    fn file(&self, path: &Path) -> FileResult<Buffer> {
        self.slot(path)?
            .buffer
            .get_or_init(|| read(path).map(Buffer::from))
            .clone()
    }
}

impl TypstSystemWorld {
    fn slot<P: AsRef<Path>>(&self, path: P) -> FileResult<MappedRwLockWriteGuard<PathSlot>> {
        let path = path.as_ref();

        let mut hashes = self.hashes.write();
        let hash = match hashes.get(path).cloned() {
            Some(hash) => hash,
            None => {
                let hash = PathHash::new(path);
                if let Ok(canon) = path.canonicalize() {
                    hashes.insert(canon.normalize(), hash.clone());
                }
                hashes.insert(path.into(), hash.clone());
                hash
            }
        }?;

        Ok(RwLockWriteGuard::map(self.paths.write(), |path| {
            path.entry(hash).or_default()
        }))
    }

    fn insert<P: AsRef<Path>>(&self, path: P, text: String) -> SourceId {
        let path = path.as_ref();

        let id = SourceId::from_u16(self.sources.len() as u16);
        let source = Source::new(id, path, text);
        self.sources.push(source);
        id
    }

    pub fn resolve_with<P: AsRef<Path>>(&self, path: P, content: &String) -> FileResult<SourceId> {
        let path = path.as_ref();
        self.slot(path)?
            .source
            .get_or_init(|| Ok(self.insert(path, content.to_string())))
            .clone()
    }

    pub fn dependant<P: AsRef<Path>>(&self, path: P) -> bool {
        let path = path.as_ref();

        if self.hashes.read().contains_key(&path.normalize()) {
            return true;
        }

        let paths = self.paths.read();
        PathHash::new(path).map_or(false, |hash| paths.contains_key(&hash))
    }

    pub fn reset(&mut self) {
        self.sources = AppendOnlyVec::new();
        self.hashes.get_mut().clear();
        self.paths.get_mut().clear();
    }
}

/// Read a file.
fn read(path: &Path) -> FileResult<Vec<u8>> {
    let f = |e| FileError::from_io(e, path);
    let mut file = File::open(path).map_err(f)?;
    if file.metadata().map_err(f)?.is_file() {
        let mut data = vec![];
        file.read_to_end(&mut data).map_err(f)?;
        Ok(data)
    } else {
        Err(FileError::IsDirectory)
    }
}

impl<'a> codespan_reporting::files::Files<'a> for TypstSystemWorld {
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

/// Searches for fonts.
pub struct SystemFontSearcher {
    pub book: FontBook,
    fonts: Vec<FontSlot>,
}

impl SystemFontSearcher {
    /// Create a new, empty system searcher.
    fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    /// Add fonts that are embedded in the binary.
    fn add_embedded(&mut self) {
        let mut add = |bytes: &'static [u8]| {
            let buffer = Buffer::from_static(bytes);
            for (_, font) in Font::iter(buffer).enumerate() {
                self.book.push(font.info().clone());
                self.fonts.push(FontSlot::with_value(Some(font)));
            }
        };

        // Embed default fonts.
        add(include_bytes!("../../../assets/fonts/LinLibertine_R.ttf"));
        add(include_bytes!("../../../assets/fonts/LinLibertine_RB.ttf"));
        add(include_bytes!("../../../assets/fonts/LinLibertine_RBI.ttf"));
        add(include_bytes!("../../../assets/fonts/LinLibertine_RI.ttf"));
        add(include_bytes!("../../../assets/fonts/NewCMMath-Book.otf"));
        add(include_bytes!(
            "../../../assets/fonts/NewCMMath-Regular.otf"
        ));
        add(include_bytes!("../../../assets/fonts/DejaVuSansMono.ttf"));
        add(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-Bold.ttf"
        ));
    }

    fn search_system(&mut self) {
        let font_paths = {
            // Search for fonts in the linux system font directories.
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                let mut font_paths = vec!["/usr/share/fonts", "/usr/local/share/fonts"]
                    .iter()
                    .map(PathBuf::from)
                    .collect::<Vec<_>>();

                if let Some(dir) = dirs::font_dir() {
                    font_paths.push(dir);
                }

                font_paths
            }
            // Search for fonts in the macOS system font directories.
            #[cfg(target_os = "macos")]
            {
                let mut font_paths = vec![
                    "/Library/Fonts",
                    "/Network/Library/Fonts",
                    "/System/Library/Fonts",
                ]
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>();

                if let Some(dir) = dirs::font_dir() {
                    font_paths.push(dir);
                }

                font_paths
            }
            // Search for fonts in the Windows system font directories.
            #[cfg(windows)]
            {
                let mut font_paths = vec![];
                let windir = std::env::var("WINDIR").unwrap_or_else(|_| "C:\\Windows".to_string());

                font_paths.push(PathBuf::from(windir).join("Fonts"));

                if let Some(roaming) = dirs::config_dir() {
                    font_paths.push(roaming.join("Microsoft\\Windows\\Fonts"));
                }
                if let Some(local) = dirs::cache_dir() {
                    font_paths.push(local.join("Microsoft\\Windows\\Fonts"));
                }

                font_paths
            }
        };

        for dir in font_paths {
            self.search_dir(dir);
        }
    }

    /// Search for all fonts in a directory recursively.
    fn search_dir(&mut self, path: impl AsRef<Path>) {
        for entry in WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("ttf" | "otf" | "TTF" | "OTF" | "ttc" | "otc" | "TTC" | "OTC"),
            ) {
                self.search_file(path);
            }
        }
    }

    /// Index the fonts in the file at the given path.
    fn search_file(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if let Ok(file) = File::open(path) {
            if let Ok(mmap) = unsafe { Mmap::map(&file) } {
                for (i, info) in FontInfo::iter(&mmap).enumerate() {
                    self.book.push(info);
                    self.fonts.push(FontSlot::new(Box::new(ReadFontLoader {
                        read: LazyFile::new(path.into()),
                        index: i as u32,
                    })));
                }
            }
        }
    }
}
