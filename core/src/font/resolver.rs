use std::sync::{Arc, RwLock};

use comemo::Prehashed;
use typst::font::{Font, FontBook};

use crate::FontSlot;

use super::{FontProfile, PartialFontBook};

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
        self.fonts[idx].get_or_init()
    }
}
