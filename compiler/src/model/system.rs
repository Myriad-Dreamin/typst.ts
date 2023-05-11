use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use memmap2::Mmap;
use typst::font::{Font, FontBook, FontInfo};
use typst::util::Buffer;
use typst_ts_core::config::CompileOpts;
use typst_ts_core::font::LazyBufferFontLoader;
use walkdir::WalkDir;

use crate::font::system::LazyFile;
use crate::source_manager::{AccessModel, FileMetadata, SourceManager};
use crate::world::CompilerFeat;
use typst_ts_core::{font::FontResolverImpl, FontSlot};

pub type TypstSystemWorld = super::world::CompilerWorld<SystemCompilerFeat>;

pub struct SystemCompilerFeat;

impl CompilerFeat for SystemCompilerFeat {
    type M = SystemAccessModel;

    fn from_opts(opts: CompileOpts) -> (FontResolverImpl, SourceManager<Self::M>) {
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
        (
            FontResolverImpl::new(searcher.book, searcher.fonts),
            SourceManager::new(SystemAccessModel {}),
        )
    }
}

pub struct SystemFileMeta {
    mt: std::time::SystemTime,
    is_file: bool,
    src: std::path::PathBuf,
}

impl FileMetadata for SystemFileMeta {
    type RealPath = same_file::Handle;

    fn mtime(&mut self) -> std::time::SystemTime {
        self.mt
    }

    fn is_file(&mut self) -> bool {
        self.is_file
    }

    fn real_path(&mut self) -> std::io::Result<Self::RealPath> {
        same_file::Handle::from_path(&self.src)
    }
}

pub struct SystemAccessModel;

impl AccessModel for SystemAccessModel {
    type FM = SystemFileMeta;

    fn stat(&self, src: &Path) -> std::io::Result<Self::FM> {
        let meta = std::fs::metadata(src)?;
        Ok(SystemFileMeta {
            mt: meta.modified()?,
            is_file: meta.is_file(),
            src: src.to_owned(),
        })
    }

    fn read_all_once(&self, src: &Path, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        std::fs::File::open(src)?.read_to_end(buf)
    }
}

/// Searches for fonts.
pub struct SystemFontSearcher {
    pub book: FontBook,
    fonts: Vec<FontSlot>,
}

fn is_font_file_by_name(path: &Path) -> bool {
    matches!(
        path.extension().map(|s| {
            let chk = |n| s.eq_ignore_ascii_case(n);
            chk("ttf") || chk("otf") || chk("ttc") || chk("otc")
        }),
        Some(true),
    )
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
        let entries = WalkDir::new(path)
            .follow_links(true)
            .sort_by(|a, b| a.file_name().cmp(b.file_name()))
            .into_iter()
            // todo: error handling
            .filter_map(|e| e.ok());

        for entry in entries {
            let path = entry.path();
            if is_font_file_by_name(path) {
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
                    self.fonts
                        .push(FontSlot::new_boxed(LazyBufferFontLoader::new(
                            LazyFile::new(path.into()),
                            i as u32,
                        )));
                }
            }
        }
    }
}
