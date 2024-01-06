use crate::{
    font::{FontGlyphProvider, GlyphProvider},
    hash::Fingerprint,
};

use super::{preludes::*, text::*, GlyphLowerBuilder, VecItem};

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
    pub items: Vec<(DefId, FlatGlyphItem)>,
    pub incremental_base: usize,
}

impl From<Vec<(DefId, FlatGlyphItem)>> for IncrGlyphPack {
    fn from(items: Vec<(DefId, FlatGlyphItem)>) -> Self {
        Self {
            items,
            incremental_base: 0,
        }
    }
}

impl FromIterator<(GlyphItem, (GlyphRef, FontRef))> for IncrGlyphPack {
    fn from_iter<T: IntoIterator<Item = (GlyphItem, (GlyphRef, FontRef))>>(iter: T) -> Self {
        let glyph_provider = GlyphProvider::new(FontGlyphProvider::default());
        let glyph_lower_builder = GlyphLowerBuilder::new(&glyph_provider, true);

        let items = iter
            .into_iter()
            .map(|(glyph, glyph_id)| {
                let glyph = glyph_lower_builder.lower_glyph(&glyph);
                glyph
                    .map(|t| {
                        let t = match t {
                            GlyphItem::Image(i) => FlatGlyphItem::Image(i),
                            GlyphItem::Outline(p) => FlatGlyphItem::Outline(p),
                            _ => unreachable!(),
                        };

                        (DefId(glyph_id.1.idx as u64), t)
                    })
                    .unwrap_or_else(|| (DefId(glyph_id.1.idx as u64), FlatGlyphItem::None))
            })
            .collect::<Vec<_>>();

        Self {
            items,
            incremental_base: 0,
        }
    }
}
