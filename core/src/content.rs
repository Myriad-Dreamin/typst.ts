use serde::{Deserialize, Serialize};

/// pdf.js compatible text content definition
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    items: Vec<TextItem>,
    styles: Vec<TextStyle>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TextItem {
    /// The text content of the item.
    str: String,
    /// direction of the text
    /// possible values: ltr, rtl
    dir: String,
    /// item width in pt
    width: f64,
    /// item height in pt
    height: f64,
    /// item transform matrix
    transform: [f64; 6],

    /// reference to the style
    #[serde(rename = "fontName")]
    font_name: String,

    /// Indicating if the text content is followed by a line-break.
    #[serde(rename = "hasEOL")]
    has_eol: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// css font family
    #[serde(rename = "fontFamily")]
    font_family: String,
    /// ascender in pt
    ascent: f64,
    /// descender in pt
    descent: f64,
    /// whether the font is vertical
    vertical: bool,
}
