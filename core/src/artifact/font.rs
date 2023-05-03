use serde::Deserialize;
use serde::Serialize;
pub use typst::font::Coverage as FontCoverage;
use typst::font::Coverage;
pub use typst::font::Font as TypstFont;
pub use typst::font::FontFlags as TypstFontFlags;
pub use typst::font::FontInfo as TypstFontInfo;
use typst::font::FontVariant;

/// Properties of a single font.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct FontInfo {
    /// The typographic font family this font is part of.
    pub family: String,
    /// Properties that distinguish this font from other fonts in the same
    /// family.
    pub variant: FontVariant,
    /// Properties of the font.
    pub flags: u32,
    /// The unicode coverage of the font.
    pub coverage: Coverage,
    /// ligature coverage
    pub ligatures: Vec<(u16, String)>,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: String::default(),
            variant: FontVariant::default(),
            coverage: Coverage::from_vec(vec![]),
            flags: u32::default(),
            ligatures: Vec::default(),
        }
    }
}
