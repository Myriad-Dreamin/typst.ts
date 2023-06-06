use serde::Deserialize;
use serde::Serialize;
pub use typst::font::Coverage as FontCoverage;
use typst::font::Coverage;
pub use typst::font::Font as TypstFont;
pub use typst::font::FontFlags as TypstFontFlags;
pub use typst::font::FontInfo as TypstFontInfo;
pub use typst::font::FontMetrics as TypstFontMetrics;
use typst::font::FontVariant;

use super::geom::Em;

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
    /// The hash of the unicode coverage.
    pub coverage_hash: String,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            family: String::default(),
            variant: FontVariant::default(),
            coverage: Coverage::from_vec(vec![]),
            flags: u32::default(),
            coverage_hash: String::default(),
        }
    }
}

/// Metrics for a decorative line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineMetrics {
    /// The vertical offset of the line from the baseline. Positive goes
    /// upwards, negative downwards.
    pub position: Em,
    /// The thickness of the line.
    pub thickness: Em,
}

/// Metrics of a font.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetrics {
    /// How many font units represent one em unit.
    pub units_per_em: f64,
    /// The distance from the baseline to the typographic ascender.
    pub ascender: Em,
    /// The approximate height of uppercase letters.
    pub cap_height: Em,
    /// The approximate height of non-ascending lowercase letters.
    pub x_height: Em,
    /// The distance from the baseline to the typographic descender.
    pub descender: Em,
    /// Recommended metrics for a strikethrough line.
    pub strikethrough: LineMetrics,
    /// Recommended metrics for an underline.
    pub underline: LineMetrics,
    /// Recommended metrics for an overline.
    pub overline: LineMetrics,
}

impl From<TypstFontMetrics> for FontMetrics {
    fn from(metrics: TypstFontMetrics) -> Self {
        Self {
            units_per_em: metrics.units_per_em,
            ascender: metrics.ascender.into(),
            cap_height: metrics.cap_height.into(),
            x_height: metrics.x_height.into(),
            descender: metrics.descender.into(),
            strikethrough: LineMetrics {
                position: metrics.strikethrough.position.into(),
                thickness: metrics.strikethrough.thickness.into(),
            },
            underline: LineMetrics {
                position: metrics.underline.position.into(),
                thickness: metrics.underline.thickness.into(),
            },
            overline: LineMetrics {
                position: metrics.overline.position.into(),
                thickness: metrics.overline.thickness.into(),
            },
        }
    }
}
