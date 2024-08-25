use std::sync::Arc;

use reflexo::hash::{item_hash128, Fingerprint};
pub use reflexo::vector::ir::*;
use ttf_parser::GlyphId;
use typst::text::Font;

/// A glyph item.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum GlyphItem {
    None,

    /// Raw glyph representation.
    /// The raw glyphs is generated in lowering stage.
    Raw(Font, GlyphId),

    /// Glyphs in SVG or Bitmap format.
    Image(Arc<ImageGlyphItem>),

    /// Glyphs in path instructions, known as the "d" attribute of a
    /// `<path/>` element.
    Outline(Arc<OutlineGlyphItem>),
}

impl From<FlatGlyphItem> for GlyphItem {
    fn from(item: FlatGlyphItem) -> Self {
        match item {
            FlatGlyphItem::Image(item) => GlyphItem::Image(item),
            FlatGlyphItem::Outline(item) => GlyphItem::Outline(item),
            FlatGlyphItem::None => GlyphItem::None,
        }
    }
}

impl GlyphItem {
    #[comemo::memoize]
    pub fn get_fingerprint(&self) -> Fingerprint {
        Fingerprint::from_u128(item_hash128(self))
    }
}
