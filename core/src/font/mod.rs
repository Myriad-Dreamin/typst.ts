use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::SystemTime,
};

use comemo::Prehashed;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use typst::{
    font::{Font, FontBook, FontInfo},
    util::Buffer,
};

use crate::ReadAllOnce;

type FontMetaDict = HashMap<String, String>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontInfoItem {
    /// customized profile data
    pub meta: FontMetaDict,
    /// The informatioin of the font
    pub info: FontInfo,
}

impl FontInfoItem {
    pub fn new(info: FontInfo) -> Self {
        Self {
            meta: Default::default(),
            info,
        }
    }

    pub fn index(&self) -> Option<u32> {
        self.meta.get("index").and_then(|v| v.parse::<u32>().ok())
    }

    pub fn set_index(&mut self, v: u32) {
        self.meta.insert("index".to_owned(), v.to_string());
    }

    pub fn meta(&self) -> &FontMetaDict {
        &self.meta
    }

    pub fn info(&self) -> &FontInfo {
        &self.info
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontProfileItem {
    /// The hash of the file
    pub hash: String,
    /// customized profile data
    pub meta: FontMetaDict,
    /// The informatioin of the font
    pub info: Vec<FontInfoItem>,
}

fn to_micro_lossy(t: SystemTime) -> u128 {
    t.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

impl FontProfileItem {
    pub fn new(kind: &str, hash: String) -> Self {
        let mut meta: FontMetaDict = Default::default();
        meta.insert("kind".to_owned(), kind.to_string());

        Self {
            hash,
            meta,
            info: Default::default(),
        }
    }

    pub fn path(&self) -> Option<&String> {
        self.meta.get("path")
    }

    pub fn mtime(&self) -> Option<SystemTime> {
        self.meta.get("mtime").and_then(|v| {
            let v = v.parse::<u64>().ok();
            v.map(|v| SystemTime::UNIX_EPOCH + std::time::Duration::from_micros(v))
        })
    }

    pub fn mtime_is_exact(&self, t: SystemTime) -> bool {
        self.mtime()
            .map(|s| {
                let s = to_micro_lossy(s);
                let t = to_micro_lossy(t);
                s == t
            })
            .unwrap_or_default()
    }

    pub fn set_path(&mut self, v: String) {
        self.meta.insert("path".to_owned(), v);
    }

    pub fn set_mtime(&mut self, v: SystemTime) {
        self.meta
            .insert("mtime".to_owned(), to_micro_lossy(v).to_string());
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn meta(&self) -> &FontMetaDict {
        &self.meta
    }

    pub fn info(&self) -> &[FontInfoItem] {
        &self.info
    }

    pub fn add_info(&mut self, info: FontInfoItem) {
        self.info.push(info);
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct FontProfile {
    pub version: String,
    pub build_info: String,
    pub items: Vec<FontProfileItem>,
}

/// A FontLoader would help load a font from somewhere.
pub trait FontLoader {
    fn load(&mut self) -> Option<Font>;
}

/// A FontResolver can resolve a font by index.
/// It also reuse FontBook for font-related query.
/// The index is the index of the font in the `FontBook.infos`.
pub trait FontResolver {
    fn font_book(&self) -> &Prehashed<FontBook>;
    fn font(&self, idx: usize) -> Option<Font>;
}

#[derive(Default)]
pub struct PartialFontBook {
    pub partial_hit: bool,
}

/// The default FontResolver implementation.
pub struct FontResolverImpl {
    book: Prehashed<FontBook>,
    partial_book: Arc<RwLock<PartialFontBook>>,
    fonts: Vec<FontSlot>,
    profile: FontProfile,
}

impl FontResolverImpl {
    pub fn new(
        book: FontBook,
        partial_book: Arc<RwLock<PartialFontBook>>,
        fonts: Vec<FontSlot>,
        profile: FontProfile,
    ) -> Self {
        Self {
            book: Prehashed::new(book),
            partial_book,
            fonts,
            profile,
        }
    }

    pub fn profile(&self) -> &FontProfile {
        &self.profile
    }

    pub fn partial_resolved(&self) -> bool {
        self.partial_book.read().unwrap().partial_hit
    }
}

impl FontResolver for FontResolverImpl {
    fn font_book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, idx: usize) -> Option<Font> {
        self.fonts[idx].get()
    }
}

/// Lazy Font Reference, load as needed.
pub struct FontSlot(
    /// reference to the loaded font
    OnceCell<Option<Font>>,
    /// reference to the font loader
    Arc<Mutex<Option<Box<dyn FontLoader>>>>,
);

impl FontSlot {
    pub fn with_value(f: Option<Font>) -> Self {
        Self(OnceCell::with_value(f), Arc::new(Mutex::new(None)))
    }

    pub fn new(f: Box<dyn FontLoader>) -> Self {
        Self(OnceCell::new(), Arc::new(Mutex::new(Some(f))))
    }

    pub fn new_boxed<F: FontLoader + 'static>(f: F) -> Self {
        Self::new(Box::new(f))
    }

    fn load(&self) -> Option<Font> {
        let mut init_fn = self.1.lock().unwrap();
        init_fn
            .take()
            .expect("the initialization fn is already poisoned")
            .load()
    }

    pub fn get(&self) -> Option<Font> {
        self.0.get_or_init(|| self.load()).clone()
    }
}

/// Load font from a buffer.
pub struct BufferFontLoader {
    pub buffer: Option<Buffer>,
    pub index: u32,
}

impl FontLoader for BufferFontLoader {
    fn load(&mut self) -> Option<Font> {
        Font::new(self.buffer.take().unwrap(), self.index)
    }
}

pub struct LazyBufferFontLoader<R> {
    pub read: Option<R>,
    pub index: u32,
}

impl<R: ReadAllOnce + Sized> LazyBufferFontLoader<R> {
    pub fn new(read: R, index: u32) -> Self {
        Self {
            read: Some(read),
            index,
        }
    }
}

impl<R: ReadAllOnce + Sized> FontLoader for LazyBufferFontLoader<R> {
    fn load(&mut self) -> Option<Font> {
        let mut buf = vec![];
        self.read.take().unwrap().read_all(&mut buf).ok()?;
        Font::new(buf.into(), self.index)
    }
}
