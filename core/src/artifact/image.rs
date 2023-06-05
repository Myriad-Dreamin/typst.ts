use crate::artifact_ir::geom::TypstAxes;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::serde_as;
pub use typst::image::Image as TypstImage;
pub use typst::image::ImageFormat;
pub use typst::image::RasterFormat;
pub use typst::image::VectorFormat;

/// A raster or vector image.
///
/// Values of this type are cheap to clone and hash.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Image {
    /// The raw, undecoded image data, follow btoa (standard) encoding
    #[serde_as(as = "Base64")]
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

impl From<TypstImage> for Image {
    fn from(image: TypstImage) -> Self {
        Image {
            data: image.data().to_vec(),
            format: match image.format() {
                ImageFormat::Raster(r) => match r {
                    RasterFormat::Png => "png",
                    RasterFormat::Jpg => "jpg",
                    RasterFormat::Gif => "gif",
                },
                ImageFormat::Vector(v) => match v {
                    VectorFormat::Svg => "svg",
                },
            }
            .to_string(),
            width: image.width(),
            height: image.height(),
            alt: image.alt().map(|s| s.to_string()),
        }
    }
}

impl From<Image> for TypstImage {
    fn from(image: Image) -> Self {
        TypstImage::new_raw(
            image.data.clone().into(),
            match image.format.as_str() {
                "png" => ImageFormat::Raster(RasterFormat::Png),
                "jpg" => ImageFormat::Raster(RasterFormat::Jpg),
                "gif" => ImageFormat::Raster(RasterFormat::Gif),
                "svg" => ImageFormat::Vector(VectorFormat::Svg),
                _ => ImageFormat::Raster(RasterFormat::Png),
            },
            TypstAxes {
                x: image.width,
                y: image.height,
            },
            image.alt.clone().map(|s| s.into()),
        )
        .unwrap()
    }
}

impl Image {
    pub fn decode_data(b: &String) -> Result<Vec<u8>, String> {
        BASE64_STANDARD.decode(b).map_err(|e| e.to_string())
    }
}
