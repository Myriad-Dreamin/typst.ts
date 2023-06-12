use crate::TypstRenderer;

use typst_ts_compiler::font::web::BrowserFontSearcher;

use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "build_glyph_pack")]
pub mod glyph_pack;

#[cfg(feature = "build_raw_font")]
pub mod raw_font;

#[cfg(feature = "build_web_font")]
pub mod web_font;

#[wasm_bindgen]
pub struct TypstRendererBuilder {
    searcher: BrowserFontSearcher,
}

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
///
#[wasm_bindgen]
impl TypstRendererBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ZResult<TypstRendererBuilder> {
        console_error_panic_hook::set_once();
        Ok(Self {
            searcher: BrowserFontSearcher::new(),
        })
    }

    pub async fn build(self) -> ZResult<TypstRenderer> {
        Ok(TypstRenderer::new(self.searcher.into()))
    }
}
