use serde::Deserialize;
use serde::Serialize;
pub use typst::image::Image as TypstImage;
pub use typst::image::ImageFormat;
pub use typst::image::RasterFormat;
pub use typst::image::VectorFormat;

/// A raster or vector image.
///
/// Values of this type are cheap to clone and hash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// The raw, undecoded image data.
    pub data: Vec<u8>,
    /// The format of the encoded `buffer`.
    pub format: String,
    /// The width in pixels.
    pub width: u32,
    /// The height in pixels.
    pub height: u32,
    /// A text describing the image.
    pub alt: Option<String>,
}
