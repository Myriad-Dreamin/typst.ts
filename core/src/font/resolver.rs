use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use comemo::Prehashed;
use typst::{
    font::{Font, FontBook, FontInfo},
    util::Buffer,
};

use crate::FontSlot;

use super::{BufferFontLoader, FontProfile, PartialFontBook};

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

    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();
        for (idx, slot) in self.fonts.iter().enumerate() {
            s.push_str(&format!("{:?} -> {:?}\n", idx, slot.get_uninitialized()));
        }
        s
    }

    pub fn profile(&self) -> &FontProfile {
        &self.profile
    }

    pub fn partial_resolved(&self) -> bool {
        self.partial_book.read().unwrap().partial_hit
    }

    pub fn loaded_fonts(&self) -> impl Iterator<Item = (usize, Font)> + '_ {
        self.fonts.iter().enumerate().flat_map(|(f, ff)| {
            ff.get_uninitialized()
                .and_then(|ff| ff.clone().map(|ff| (f, ff.clone())))
        })
    }

    pub fn modify_font_data(&mut self, idx: usize, buffer: Buffer) {
        let mut font_book: std::sync::RwLockWriteGuard<PartialFontBook> =
            self.partial_book.write().unwrap();
        for (i, info) in FontInfo::iter(buffer.as_slice()).enumerate() {
            let buffer = buffer.clone();
            let modify_idx = if i > 0 { None } else { Some(idx) };

            font_book.push((
                modify_idx,
                info,
                FontSlot::new(Box::new(BufferFontLoader {
                    buffer: Some(buffer),
                    index: i as u32,
                })),
            ));
        }
    }

    pub fn rebuild(&mut self) {
        let mut partial_book = self.partial_book.write().unwrap();
        if !partial_book.partial_hit {
            return;
        }

        let mut book = FontBook::default();

        let mut font_changes = HashMap::new();
        let mut new_fonts = vec![];
        for (idx, info, slot) in partial_book.changes.drain(..) {
            if let Some(idx) = idx {
                font_changes.insert(idx, (info, slot));
            } else {
                new_fonts.push((info, slot));
            }
        }
        partial_book.changes.clear();
        partial_book.partial_hit = false;

        let mut font_slots = Vec::new();
        font_slots.append(&mut self.fonts);
        self.fonts.clear();

        for i in 0..font_slots.len() {
            let (info, slot) = if let Some((_, v)) = font_changes.remove_entry(&i) {
                v
            } else {
                book.push(self.book.info(i).unwrap().clone());
                continue;
            };

            book.push(info);
            font_slots[i] = slot;
        }

        for (info, slot) in new_fonts.drain(..) {
            book.push(info);
            font_slots.push(slot);
        }

        self.book = Prehashed::new(book);
        self.fonts = font_slots;
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
