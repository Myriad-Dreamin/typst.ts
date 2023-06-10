use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex, RwLock},
};

use comemo::Prehashed;
use ttf_parser::GlyphId;
use typst::{
    font::{Font, FontBook, FontFlags, FontInfo},
    util::Buffer,
};

use crate::{artifact::image::TypstImage, FontSlot};

use super::{
    BufferFontLoader, FontGlyphPackBundle, FontInfoKey, FontProfile, GlyphProvider, IGlyphProvider,
    PartialFont, PartialFontBook,
};

use std::hash::{Hash, Hasher};

/// A FontResolver can resolve a font by index.
/// It also reuse FontBook for font-related query.
/// The index is the index of the font in the `FontBook.infos`.
pub trait FontResolver {
    fn font_book(&self) -> &Prehashed<FontBook>;
    fn font(&self, idx: usize) -> Option<Font>;
    // todo: reference or arc
    fn partial_font(&self, _font: &Font) -> Option<&PartialFont> {
        None
    }

    fn default_get_by_info(&self, info: &FontInfo) -> Option<Font> {
        // todo: font alternative
        let mut alternative_text = 'c';
        if let Some(codepoint) = info.coverage.iter().next() {
            alternative_text = std::char::from_u32(codepoint).unwrap();
        };

        let idx = self
            .font_book()
            .select_fallback(Some(info), info.variant, &alternative_text.to_string())
            .unwrap();
        self.font(idx)
    }
    fn get_by_info(&self, info: &FontInfo) -> Option<Font> {
        self.default_get_by_info(info)
    }
}

/// The default FontResolver implementation.
pub struct FontResolverImpl {
    book: Prehashed<FontBook>,
    partial_book: Arc<Mutex<PartialFontBook>>,
    fonts: Vec<FontSlot>,
    profile: FontProfile,
    pub font_map: HashMap<FontInfoKey, PartialFont>,
}

impl FontResolverImpl {
    pub fn new(
        book: FontBook,
        partial_book: Arc<Mutex<PartialFontBook>>,
        fonts: Vec<FontSlot>,
        profile: FontProfile,
        font_map: HashMap<FontInfoKey, PartialFont>,
    ) -> Self {
        Self {
            book: Prehashed::new(book),
            partial_book,
            fonts,
            profile,
            font_map,
        }
    }

    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    pub fn profile(&self) -> &FontProfile {
        &self.profile
    }

    pub fn partial_resolved(&self) -> bool {
        self.partial_book.lock().unwrap().partial_hit
    }

    pub fn loaded_fonts(&self) -> impl Iterator<Item = (usize, Font)> + '_ {
        let slots_with_index = self.fonts.iter().enumerate();

        slots_with_index.flat_map(|(idx, slot)| {
            let maybe_font = slot.get_uninitialized().flatten();
            maybe_font.map(|font| (idx, font))
        })
    }

    pub fn modify_font_data(&mut self, idx: usize, buffer: Buffer) {
        let mut font_book = self.partial_book.lock().unwrap();
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

    pub fn append_font(&mut self, info: FontInfo, slot: FontSlot) {
        let mut font_book = self.partial_book.lock().unwrap();
        font_book.push((None, info, slot));
    }

    pub fn rebuild(&mut self) {
        let mut partial_book = self.partial_book.lock().unwrap();
        if !partial_book.partial_hit {
            return;
        }
        partial_book.revision += 1;

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

        for (i, slot_ref) in font_slots.iter_mut().enumerate() {
            let (info, slot) = if let Some((_, v)) = font_changes.remove_entry(&i) {
                v
            } else {
                book.push(self.book.info(i).unwrap().clone());
                continue;
            };

            book.push(info);
            *slot_ref = slot;
        }

        for (info, slot) in new_fonts.drain(..) {
            book.push(info);
            font_slots.push(slot);
        }

        self.book = Prehashed::new(book);
        self.fonts = font_slots;
    }

    pub fn add_glyph_packs(&mut self, font_glyph_bundle: FontGlyphPackBundle) {
        let mut changes = Vec::new();

        for font_glyphs in font_glyph_bundle.fonts {
            let key = FontInfoKey {
                family: font_glyphs.info.family.clone(),
                variant: font_glyphs.info.variant,
                flags: FontFlags::from_bits(font_glyphs.info.flags).unwrap(),
            };

            let glyphs = || font_glyphs.glyphs.into_iter().map(|g| (GlyphId(g.id), g));

            match self.font_map.entry(key) {
                Entry::Occupied(mut entry) => {
                    let font = entry.get_mut();
                    font.glyphs.extend(glyphs());
                }
                Entry::Vacant(entry) => {
                    let font = Font::new_dummy(
                        font_glyphs.info.clone().into(),
                        font_glyphs.metrics.into(),
                    )
                    .unwrap();

                    changes.push((
                        None,
                        font.info().clone(),
                        FontSlot::with_value(Some(font.clone())),
                    ));

                    entry.insert(PartialFont {
                        typst_repr: font,
                        glyphs: glyphs().collect(),
                    });
                }
            }
        }

        if !changes.is_empty() {
            let mut partial_book = self.partial_book.lock().unwrap();
            partial_book.partial_hit = true;
            partial_book.changes.extend(changes);

            drop(partial_book);
            self.rebuild();
        }
    }
}

impl FontResolver for FontResolverImpl {
    fn font_book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn font(&self, idx: usize) -> Option<Font> {
        self.fonts[idx].get_or_init()
    }

    fn partial_font(&self, font: &Font) -> Option<&PartialFont> {
        self.font_map.get(&font.info().into())
    }

    fn get_by_info(&self, info: &FontInfo) -> Option<Font> {
        if let Some(font) = self.font_map.get(&info.into()) {
            return Some(font.typst_repr.clone());
        }

        FontResolver::default_get_by_info(self, info)
    }
}

impl fmt::Display for FontResolverImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, slot) in self.fonts.iter().enumerate() {
            writeln!(f, "{:?} -> {:?}", idx, slot.get_uninitialized())?;
        }

        Ok(())
    }
}

pub struct PartialFontGlyphProvider {
    base: GlyphProvider,
    resolver: Arc<RwLock<FontResolverImpl>>,
}

// todo: appropriate hash
impl Hash for PartialFontGlyphProvider {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(11123111);
        let ptr = self.resolver.as_ref() as *const _;
        state.write_usize(ptr as usize);
    }
}

impl PartialFontGlyphProvider {
    pub fn new(base: GlyphProvider, resolver: Arc<RwLock<FontResolverImpl>>) -> Self {
        Self { base, resolver }
    }
}

impl IGlyphProvider for PartialFontGlyphProvider {
    fn svg_glyph(&self, font: &Font, id: GlyphId) -> Option<Arc<[u8]>> {
        if !font.is_dummy() {
            return self.base.svg_glyph(font, id);
        }

        let partial_book = self.resolver.read().unwrap();
        let glyph = partial_book
            .font_map
            .get(&font.info().into())?
            .glyphs
            .get(&id)?;
        Some(glyph.svg.as_ref()?.image.clone())
    }

    fn bitmap_glyph(&self, font: &Font, id: GlyphId, ppem: u16) -> Option<(TypstImage, i16, i16)> {
        if !font.is_dummy() {
            return self.base.bitmap_glyph(font, id, ppem);
        }

        let partial_book = self.resolver.read().unwrap();
        let glyph = partial_book
            .font_map
            .get(&font.info().into())?
            .glyphs
            .get(&id)?;
        glyph
            .bitmap
            .as_ref()
            .map(|info| (info.image.clone().into(), info.x, info.y))
    }

    fn outline_glyph(&self, font: &Font, id: GlyphId) -> Option<String> {
        if !font.is_dummy() {
            return self.base.outline_glyph(font, id);
        }

        let partial_book = self.resolver.read().unwrap();
        let glyph = partial_book
            .font_map
            .get(&font.info().into())?
            .glyphs
            .get(&id)?;
        Some(glyph.outline.as_ref()?.outline.clone())
    }
}

#[cfg(test)]
mod tests {
    use typst::{
        font::FontVariant,
        font::{Coverage, FontFlags, FontInfo},
        font::{Font, FontMetrics, LineMetrics},
        geom::Em,
    };

    #[test]
    fn test_dummy_typst_font() {
        let font_info = FontInfo {
            family: "Crazy Font Roman".to_string(),
            variant: FontVariant::default(),
            flags: FontFlags::empty(),
            coverage: Coverage::from_vec(vec![]),
        };
        let font_metric = FontMetrics {
            units_per_em: 1.,
            ascender: Em::new(1.),
            cap_height: Em::new(1.),
            x_height: Em::new(1.),
            descender: Em::new(1.),
            strikethrough: LineMetrics {
                position: Em::new(1.),
                thickness: Em::new(1.),
            },
            underline: LineMetrics {
                position: Em::new(1.),
                thickness: Em::new(1.),
            },
            overline: LineMetrics {
                position: Em::new(1.),
                thickness: Em::new(1.),
            },
        };
        let font = Font::new_dummy(font_info.clone(), font_metric).unwrap();

        assert!(font.is_dummy());
        assert_eq!(font.info().family, font_info.family);
    }
}
