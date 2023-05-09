use std::sync::{Arc, Mutex};

use comemo::Prehashed;
use once_cell::sync::OnceCell;
use typst::{
    font::{Font, FontBook},
    util::Buffer,
};

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

/// The default FontResolver implementation.
pub struct FontResolverImpl {
    book: Prehashed<FontBook>,
    fonts: Vec<FontSlot>,
}

impl FontResolverImpl {
    pub fn new(book: FontBook, fonts: Vec<FontSlot>) -> Self {
        Self {
            book: Prehashed::new(book),
            fonts,
        }
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
