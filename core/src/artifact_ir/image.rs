use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::Deserialize;
use serde::Serialize;
pub use typst::image::Image as TypstImage;
pub use typst::image::ImageFormat;
pub use typst::image::RasterFormat;
pub use typst::image::VectorFormat;

use super::core::ItemRef;

/// A raster or vector image.
///
/// Values of this type are cheap to clone and hash.
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// The raw, undecoded image data, follow btoa (standard) encoding
    pub data: ItemRef<Vec<u8>>,
    pub data_len: u64,
    /// The format of the encoded `buffer`.
    pub format: ItemRef<String>,
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// A text describing the image.
    pub alt: Option<ItemRef<String>>,
}

impl Image {
    pub fn decode_data(b: &String) -> Result<Vec<u8>, String> {
        BASE64_STANDARD.decode(b).map_err(|e| e.to_string())
    }
}
