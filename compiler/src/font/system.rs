use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

use memmap2::Mmap;
use sha2::{Digest, Sha256};
use typst::{
    font::{Font, FontBook, FontInfo},
    util::Buffer,
};
use typst_ts_core::{
    build_info,
    font::{FontInfoItem, FontProfile, FontProfileItem, FontResolverImpl, LazyBufferFontLoader},
    FontSlot,
};
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

#[derive(Default)]
struct FontProfileRebuilder {
    path_items: HashMap<PathBuf, FontProfileItem>,
    pub profile: FontProfile,
    can_profile: bool,
}

impl FontProfileRebuilder {
    /// Index the fonts in the file at the given path.
    pub fn search_file(&mut self, path: impl AsRef<Path>) -> Option<&FontProfileItem> {
        let path = path.as_ref().canonicalize().unwrap();
        if let Some(item) = self.path_items.get(&path) {
            return Some(item);
        }

        if let Ok(mut file) = File::open(&path) {
            let hash = if self.can_profile {
                let mut hasher = Sha256::new();
                let _bytes_written = std::io::copy(&mut file, &mut hasher).unwrap();
                let hash = hasher.finalize();

                format!("sha256:{}", hex::encode(hash))
            } else {
                "".to_owned()
            };

            let mut profile_item = FontProfileItem::new("path", hash);
            profile_item.set_path(path.clone().to_str().unwrap().to_owned());
            profile_item.set_mtime(file.metadata().unwrap().modified().unwrap());

            // println!("searched font: {:?}", path);

            if let Ok(mmap) = unsafe { Mmap::map(&file) } {
                for (i, info) in FontInfo::iter(&mmap).enumerate() {
                    let mut ff = FontInfoItem::new(info);
                    if i != 0 {
                        ff.set_index(i as u32);
                    }
                    profile_item.add_info(ff);
                }
            }

            self.profile.items.push(profile_item);
            return self.profile.items.last();
        }

        None
    }
}

/// Searches for fonts.
pub struct SystemFontSearcher {
    pub book: FontBook,
    pub fonts: Vec<FontSlot>,
    profile_rebuilder: FontProfileRebuilder,
}

impl SystemFontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        let mut profile_rebuilder = FontProfileRebuilder::default();
        profile_rebuilder.profile.version = "v1beta".to_owned();
        profile_rebuilder.profile.build_info = build_info::VERSION.to_string();

        Self {
            book: FontBook::new(),
            fonts: vec![],
            profile_rebuilder,
        }
    }

    pub fn set_can_profile(&mut self, can_profile: bool) {
        self.profile_rebuilder.can_profile = can_profile;
    }

    pub fn add_profile_by_path(&mut self, profile_path: &Path) {
        // let begin = std::time::Instant::now();
        // profile_path is in format of json.gz
        let profile_file = File::open(profile_path).unwrap();
        let profile_gunzip = flate2::read::GzDecoder::new(profile_file);
        let profile: FontProfile = serde_json::from_reader(profile_gunzip).unwrap();

        if self.profile_rebuilder.profile.version != profile.version
            || self.profile_rebuilder.profile.build_info != profile.build_info
        {
            return;
        }

        for item in profile.items {
            let path = match item.path() {
                Some(path) => path,
                None => continue,
            };
            let path = PathBuf::from(path);

            if let Ok(m) = std::fs::metadata(&path) {
                let modified = m.modified().ok();
                if !modified.map(|m| item.mtime_is_exact(m)).unwrap_or_default() {
                    continue;
                }
            }

            self.profile_rebuilder.path_items.insert(path, item.clone());
            self.profile_rebuilder.profile.items.push(item);
        }
        // let end = std::time::Instant::now();
        // println!("profile_rebuilder init took {:?}", end - begin);
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
        let profile_item = match self.profile_rebuilder.search_file(path.as_ref()) {
            Some(profile_item) => profile_item,
            None => return,
        };

        for info in profile_item.info.iter() {
            self.book.push(info.info.clone());
            let i = info.index().unwrap_or_default();
            self.fonts
                .push(FontSlot::new_boxed(LazyBufferFontLoader::new(
                    LazyFile::new(path.as_ref().to_owned()),
                    i as u32,
                )));
        }
    }
}

impl Default for SystemFontSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl From<SystemFontSearcher> for FontResolverImpl {
    fn from(searcher: SystemFontSearcher) -> Self {
        FontResolverImpl::new(
            searcher.book,
            searcher.fonts,
            searcher.profile_rebuilder.profile,
        )
    }
}
