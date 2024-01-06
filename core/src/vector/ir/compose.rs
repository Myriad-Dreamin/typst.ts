use crate::hash::Fingerprint;

use super::{preludes::*, text::*, VecItem};

/// Flatten mapping fingerprints to vector items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct ItemPack(pub Vec<(Fingerprint, VecItem)>);

/// Flatten mapping fingerprints to glyph items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct IncrFontPack {
    pub items: Vec<FontItem>,
    pub incremental_base: usize,
}

impl From<Vec<FontItem>> for IncrFontPack {
    fn from(items: Vec<FontItem>) -> Self {
        Self {
            items,
            incremental_base: 0,
        }
    }
}

/// Flatten mapping fingerprints to glyph items.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "rkyv", derive(Archive, rDeser, rSer))]
#[cfg_attr(feature = "rkyv-validation", archive(check_bytes))]
pub struct IncrGlyphPack {
    pub items: Vec<(GlyphRef, FlatGlyphItem)>,
    pub incremental_base: usize,
}

impl From<Vec<(GlyphRef, FlatGlyphItem)>> for IncrGlyphPack {
    fn from(items: Vec<(GlyphRef, FlatGlyphItem)>) -> Self {
        Self {
            items,
            incremental_base: 0,
        }
    }
}

impl FromIterator<(GlyphRef, FlatGlyphItem)> for IncrGlyphPack {
    fn from_iter<T: IntoIterator<Item = (GlyphRef, FlatGlyphItem)>>(iter: T) -> Self {
        Self {
            items: iter.into_iter().collect(),
            incremental_base: 0,
        }
    }
}
