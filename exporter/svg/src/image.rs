use crate::utils::AbsExt;
use crate::{RenderFeature, SvgRenderTask};
use base64::Engine;
use typst::geom::Size;
use typst::image::{Image, ImageFormat, RasterFormat, VectorFormat};
use typst_ts_core::error::prelude::ZResult;

impl<Feat: RenderFeature> SvgRenderTask<Feat> {
    /// Render a raster or SVG image into the canvas.
    // todo: error handling
    pub(crate) fn render_image(&mut self, image: &Image, size: Size) -> ZResult<String> {
        let _r = self.perf_event("render_image");

        let _l = self.perf_event("load_image");
        let image_url = rasterize_embedded_image_url(image).unwrap();

        // resize image to fit the view
        let size = size;
        let view_width = size.x.to_f32();
        let view_height = size.y.to_f32();

        let aspect = (image.width() as f32) / (image.height() as f32);

        let w = view_width.max(aspect * view_height);
        let h = w / aspect;
        Ok(format!(
            r#"<image x="0" y="0" width="{}" height="{}" xlink:href="{}" />"#,
            w, h, image_url
        ))
    }
}

#[derive(Debug, Clone)]
struct ImageUrl(String);

#[cfg(feature = "web")]
impl Drop for ImageUrl {
    fn drop(&mut self) {
        web_sys::Url::revoke_object_url(&self.0).unwrap();
    }
}

#[comemo::memoize]
#[cfg(feature = "web")]
fn rasterize_image_url(image: &Image) -> Option<Arc<ImageUrl>> {
    let u = js_sys::Uint8Array::new_with_length(image.data().len() as u32);
    u.copy_from(image.data());

    let parts = js_sys::Array::new();
    parts.push(&u);
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
        &parts,
        web_sys::BlobPropertyBag::new().type_(match image.format() {
            ImageFormat::Raster(e) => match e {
                RasterFormat::Jpg => "image/jpeg",
                RasterFormat::Png => "image/png",
                RasterFormat::Gif => "image/gif",
            },
            ImageFormat::Vector(e) => match e {
                // todo: security check
                // https://security.stackexchange.com/questions/148507/how-to-prevent-xss-in-svg-file-upload
                // todo: use our custom font
                VectorFormat::Svg => "image/svg+xml",
            },
        }),
    )
    .unwrap();

    // todo: memory leak
    let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

    Some(Arc::new(ImageUrl(data_url)))
}

fn rasterize_embedded_image_url(image: &Image) -> Option<String> {
    let url = match image.format() {
        ImageFormat::Raster(e) => match e {
            RasterFormat::Jpg => "data:image/jpeg;base64,",
            RasterFormat::Png => "data:image/png;base64,",
            RasterFormat::Gif => "data:image/gif;base64,",
        },
        ImageFormat::Vector(e) => match e {
            VectorFormat::Svg => "data:image/svg+xml;base64,",
        },
    };

    let mut data = base64::engine::general_purpose::STANDARD.encode(image.data());
    data.insert_str(0, url);
    Some(data)
}
