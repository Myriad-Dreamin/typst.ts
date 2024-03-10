#[cfg(feature = "embedded-fonts")]
pub fn fonts() -> impl Iterator<Item = &'static [u8]> {
    typst_assets::fonts()
}

#[cfg(not(feature = "embedded-fonts"))]
pub fn fonts() -> impl Iterator<Item = &'static [u8]> {
    static EMBEDDED_FONT: &[&[u8]] = &[];
    EMBEDDED_FONT.iter().copied()
}
