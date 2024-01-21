use serde::{Deserialize, Serialize};

/// pdf.js compatible text content definition
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TextContent {
    pub items: Vec<TextItem>,
    pub styles: Vec<TextStyle>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TextItem {
    /// The text content of the item.
    pub str: String,
    /// direction of the text
    /// possible values: ltr, rtl, ttb, btt
    pub dir: String,
    /// item width in pt
    pub width: f32,
    /// item height in pt
    pub height: f32,
    /// item transform matrix
    pub transform: [f32; 6],

    /// reference to the style
    #[serde(rename = "fontName")]
    pub font_name: u32,

    /// Indicating if the text content is followed by a line-break.
    #[serde(rename = "hasEOL")]
    pub has_eol: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    /// css font family
    #[serde(rename = "fontFamily")]
    pub font_family: String,
    /// ascender in pt
    pub ascent: f32,
    /// descender in pt
    pub descent: f32,
    /// whether the font is vertical
    pub vertical: bool,
}
