use crate::TypstCompiler;

use typst_ts_compiler::font::web::{BrowserFontSearcher, WebFont};

use js_sys::Uint8Array;
use typst::util::Buffer;
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub struct TypstCompilerBuilder {
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstCompilerBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstCompilerBuilder, JsValue> {
        console_error_panic_hook::set_once();
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
    pub async fn add_web_font(&mut self, font: WebFont) -> Result<(), JsValue> {
        let v: JsValue = format!("web font loading: {:?}", font).into();
        console::info_1(&v);

        self.searcher.add_web_font(font).await;

        Ok(())
    }

    // 24 MB
    pub async fn build(self) -> Result<TypstCompiler, JsValue> {
        TypstCompiler::new(self.searcher).await
    }
}

impl TypstCompilerBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Buffer) {
        self.searcher.add_font_data(font_buffer);
    }
}
