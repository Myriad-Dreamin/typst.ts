use crate::TypstRenderer;

use typst_ts_compiler::font::web::BrowserFontSearcher;

use js_sys::Uint8Array;
use typst::util::Buffer;
use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TypstRendererBuilder {
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstRendererBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ZResult<TypstRendererBuilder> {
        Ok(Self {
            searcher: BrowserFontSearcher::new(),
        })
    }

    // 400 KB
    pub async fn add_raw_font(&mut self, font_buffer: Uint8Array) -> ZResult<()> {
        // let v: JsValue =
        //     format!("raw font loading: Buffer({:?})", font_buffer.byte_length()).into();
        // console::info_1(&v);

        self.add_raw_font_internal(font_buffer.to_vec().into());
        Ok(())
    }

    // 100 KB
    pub async fn add_web_fonts(&mut self, font: js_sys::Array) -> ZResult<()> {
        self.searcher.add_web_fonts(font).await
    }

    // 24 MB
    pub async fn build(self) -> ZResult<TypstRenderer> {
        Ok(TypstRenderer::new(self.searcher.into()))
    }
}

impl TypstRendererBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Buffer) {
        self.searcher.add_font_data(font_buffer);
    }
}
