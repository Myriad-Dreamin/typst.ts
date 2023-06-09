#[cfg(feature = "render_canvas")]
pub mod canvas;

#[cfg(feature = "render_pdf")]
pub mod pdf;

#[cfg(feature = "render_raster")]
pub mod raster;

#[cfg(not(feature = "render_raster"))]
pub mod raster_stub {
    #![allow(dead_code)]
    #![allow(unused_imports)]

    use typst_ts_core::error::prelude::*;
    use wasm_bindgen::prelude::*;
    use web_sys::ImageData;

    use crate::{RenderPageImageOptions, RenderSession, TypstRenderer};

    #[wasm_bindgen]
    impl TypstRenderer {
        pub fn render_page(
            &mut self,
            _session: &RenderSession,
            _options: Option<RenderPageImageOptions>,
        ) -> ZResult<ImageData> {
            Err(error_once!("Renderer.RasterFeatureNotEnabled"))
        }
    }
}
#[cfg(not(feature = "render_raster"))]
pub use raster_stub::*;
