use crate::TypstRenderer;

use typst_ts_compiler::font::web::BrowserFontSearcher;

use js_sys::Uint8Array;
use typst::util::Buffer;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct TypstRendererBuilder {
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstRendererBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstRendererBuilder, JsValue> {
        Ok(Self {
            searcher: BrowserFontSearcher::new(),
        })
    }

    // 400 KB
    pub async fn add_raw_font(&mut self, font_buffer: Uint8Array) -> Result<(), JsValue> {
        // let v: JsValue =
        //     format!("raw font loading: Buffer({:?})", font_buffer.byte_length()).into();
        // console::info_1(&v);

        self.add_raw_font_internal(font_buffer.to_vec().into());
        Ok(())
    }

    // 100 KB
    pub async fn add_web_fonts(&mut self, font: js_sys::Array) -> Result<(), JsValue> {
        let v: JsValue = format!("web font loading: {:?}", font).into();
        console::info_1(&v);

        self.searcher.add_web_fonts(font).await;

        Ok(())
    }

    // 24 MB
    pub async fn build(self) -> Result<TypstRenderer, JsValue> {
        Ok(TypstRenderer::new(self.searcher.into()))
    }
}

impl TypstRendererBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Buffer) {
        self.searcher.add_font_data(font_buffer);
    }
}
