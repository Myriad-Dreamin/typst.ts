const WOFF2_SIGNATURE: &[u8; 4] = b"wOF2";

/// Decodes supported web font containers into OpenType font data.
pub fn decode_font_data(data: Vec<u8>) -> Result<Vec<u8>, String> {
    if !data.starts_with(WOFF2_SIGNATURE) {
        return Ok(data);
    }

    wuff::decompress_woff2(&data).map_err(|err| format!("failed to decode WOFF2 font: {err:?}"))
}
