#[cfg(feature = "build_raw_font")]
pub mod raw_font;

use crate::TypstRenderer;

use reflexo_typst::error::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TypstRendererBuilder {}

/// A builder for [`TypstRenderer`].
/// The builder is used to configure the renderer before building it.
/// - configure fonts for rendering
///   - configure with `glyph_pack`
///   - configure with `raw_font`
///   - configure with `web_font`
/// Example usage:
/// ```js
/// const builder = new TypstRendererBuilder();
/// const renderer = await builder.build();
/// ```
#[wasm_bindgen]
impl TypstRendererBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstRendererBuilder> {
        console_error_panic_hook::set_once();
        Ok(Self {})
    }

    pub async fn build(self) -> Result<TypstRenderer> {
        Ok(TypstRenderer::new())
    }
}

#[cfg(not(feature = "build_glyph_pack"))]
pub mod glyph_pack_stub {

    use std::sync::{Mutex, OnceLock};

    use reflexo_typst::error::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::{TypstRenderer, TypstRendererBuilder};

    static WARN_ONCE1: Mutex<OnceLock<()>> = Mutex::new(OnceLock::new());
    static WARN_ONCE2: Mutex<OnceLock<()>> = Mutex::new(OnceLock::new());

    #[wasm_bindgen]
    impl TypstRendererBuilder {
        pub async fn add_glyph_pack(&mut self, _pack: JsValue) -> Result<()> {
            WARN_ONCE1.lock().unwrap().get_or_init(|| {
                web_sys::console::warn_1(
                    &"[typst-ts-renderer]: build_glyph_pack feature is not enabled, calling TypstRendererBuilder::add_glyph_pack".into(),
                );
            });
            Ok(())
        }
    }

    #[wasm_bindgen]
    impl TypstRenderer {
        pub fn load_glyph_pack(&self, _v: JsValue) -> Result<()> {
            WARN_ONCE2.lock().unwrap().get_or_init(|| {
                web_sys::console::warn_1(
                    &"[typst-ts-renderer]: build_glyph_pack feature is not enabled, calling TypstRenderer::load_glyph_pack".into(),
                );
            });
            Ok(())
        }
    }
}

#[cfg(not(feature = "build_glyph_pack"))]
#[allow(unused_imports)]
pub use glyph_pack_stub::*;

#[cfg(not(feature = "build_raw_font"))]
pub mod raw_font_stub {

    use std::sync::{Mutex, OnceLock};

    use reflexo_typst::error::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::TypstRendererBuilder;

    static WARN_ONCE: Mutex<OnceLock<()>> = Mutex::new(OnceLock::new());

    #[wasm_bindgen]
    impl TypstRendererBuilder {
        pub async fn add_raw_font(&mut self, _font_buffer: js_sys::Uint8Array) -> Result<()> {
            WARN_ONCE.lock().unwrap().get_or_init(|| {
                web_sys::console::warn_1(
                    &"[typst-ts-renderer]: build_raw_font feature is not enabled".into(),
                );
            });
            Ok(())
        }
    }
}
#[cfg(not(feature = "build_raw_font"))]
#[allow(unused_imports)]
pub use raw_font_stub::*;

pub mod web_font_stub {

    use std::sync::{Mutex, OnceLock};

    use reflexo_typst::error::prelude::*;
    use wasm_bindgen::prelude::*;

    use crate::TypstRendererBuilder;

    static WARN_ONCE: Mutex<OnceLock<()>> = Mutex::new(OnceLock::new());

    #[wasm_bindgen]
    impl TypstRendererBuilder {
        pub async fn add_lazy_font(&mut self, _font: JsValue, _blob: JsValue) -> Result<()> {
            WARN_ONCE.lock().unwrap().get_or_init(|| {
                web_sys::console::warn_1(
                    &"[typst-ts-renderer]: add_lazy_font feature is removed".into(),
                );
            });
            Ok(())
        }
    }
}
