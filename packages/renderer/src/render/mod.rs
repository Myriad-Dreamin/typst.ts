#[cfg(feature = "render_canvas")]
pub mod canvas;

#[cfg(feature = "render_pdf")]
pub mod pdf;

#[cfg(feature = "render_svg")]
pub mod svg;

#[cfg(feature = "render_dom")]
pub mod dom;

#[cfg(not(feature = "render_canvas"))]
pub mod canvas_stub {
    #![allow(dead_code)]
    #![allow(unused_imports)]

    use reflexo_typst::error::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::{RenderPageImageOptions, RenderSession, TypstRenderer};

    #[wasm_bindgen]
    impl TypstRenderer {
        pub async fn render_page_to_canvas(
            &mut self,
            _ses: &RenderSession,
            _canvas: &JsValue,
            _options: Option<RenderPageImageOptions>,
        ) -> Result<JsValue> {
            Err(error_once!("Renderer.CanvasFeatureNotEnabled"))
        }
    }
}
#[cfg(not(feature = "render_canvas"))]
#[allow(unused_imports)]
pub use canvas_stub::*;
