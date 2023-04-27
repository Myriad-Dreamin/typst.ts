use serde::Deserialize;
use serde::Serialize;
pub use typst::font::Coverage as FontCoverage;
use typst::font::Coverage;
pub use typst::font::Font as TypstFont;
use typst::font::FontFlags;
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
    pub flags: FontFlags,
    /// The unicode coverage of the font.
    pub coverage: Coverage,
    /// ligature coverage
    pub ligatures: Vec<(u16, String)>,
}
