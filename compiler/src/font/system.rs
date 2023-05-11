use std::{
    fs::File,
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use typst::{
    font::{Font, FontBook, FontInfo},
    util::Buffer,
};
use typst_ts_core::{font::LazyBufferFontLoader, FontSlot};
use walkdir::WalkDir;

use crate::vfs::system::LazyFile;

fn is_font_file_by_name(path: &Path) -> bool {
    matches!(
        path.extension().map(|s| {
            let chk = |n| s.eq_ignore_ascii_case(n);
            chk("ttf") || chk("otf") || chk("ttc") || chk("otc")
        }),
        Some(true),
    )
}

/// Searches for fonts.
pub struct SystemFontSearcher {
    pub book: FontBook,
    pub fonts: Vec<FontSlot>,
}

impl SystemFontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    /// Add fonts that are embedded in the binary.
    pub fn add_embedded(&mut self) {
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

    pub fn search_system(&mut self) {
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
    pub fn search_dir(&mut self, path: impl AsRef<Path>) {
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
    pub fn search_file(&mut self, path: impl AsRef<Path>) {
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

impl Default for SystemFontSearcher {
    fn default() -> Self {
        Self::new()
    }
}
