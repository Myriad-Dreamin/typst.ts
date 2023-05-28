use crate::utils::{console_log, AbsExt, CanvasStateGuard};
use crate::{sk, CanvasRenderTask};
use typst::geom::Size;
use typst::image::{Image, ImageFormat, RasterFormat, VectorFormat};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsCast, JsValue};

impl<'a> CanvasRenderTask<'a> {
    /// Render a raster or SVG image into the canvas.
    // todo: error handling, refactor, and verify correctness
    pub(crate) fn render_image(
        &mut self,
        ts: sk::Transform,
        image: &Image,
        size: Size,
    ) -> Option<()> {
        let view_width = size.x.to_f32();
        let view_height = size.y.to_f32();

        let aspect = (image.width() as f32) / (image.height() as f32);
        let scale = ts.sx.max(ts.sy);
        let w = (scale * view_width.max(aspect * view_height)).ceil() as u32;
        let h = ((w as f32) / aspect).ceil() as u32;

        let window = web_sys::window().unwrap();

        let img = window
            .document()
            .unwrap()
            .create_element("img")
            .unwrap()
            .dyn_into::<web_sys::HtmlImageElement>()
            .unwrap();

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
                    VectorFormat::Svg => "image/svg+xml",
                },
            }),
        )
        .unwrap();

        let data_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        let remote_data_url = data_url.clone();

        let session_id = self.session_id.clone();

        let session_id2 = session_id.clone();
        let data_url2 = data_url.clone();

        let x = ts.tx;
        let y = ts.ty;

        let img_ref = img.clone();

        let a = Closure::<dyn Fn()>::new(move || {
            web_sys::Url::revoke_object_url(&remote_data_url).unwrap();

            // todo: mask
            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .query_selector(format!("canvas[data-typst-session='{}']", session_id).as_str())
                .unwrap();

            // console_log!("loaded {} {}", session_id, remote_data_url);

            let canvas = if let Some(canvas) = canvas {
                canvas
            } else {
                return;
            };

            let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            // console_log!(
            //     "ready {} {} {:?}",
            //     session_id,
            //     remote_data_url,
            //     (x, y, w, h)
            // );

            let state = CanvasStateGuard::new(&ctx);
            ctx.reset_transform().unwrap();
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &img_ref, x as f64, y as f64, w as f64, h as f64,
            )
            .unwrap();
            drop(state);
        });

        img.set_onload(Some(a.as_ref().unchecked_ref()));
        a.forget();

        let a = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| {
            console_log!(
                "err image loading {} {:?} {:?} {}",
                session_id2,
                js_sys::Reflect::get(&e, &"type".into()).unwrap(),
                js_sys::JSON::stringify(&e).unwrap(),
                data_url2,
            );
        });

        img.set_onerror(Some(a.as_ref().unchecked_ref()));
        a.forget();

        img.set_src(&data_url);

        Some(())
    }
}
