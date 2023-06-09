use crate::TypstRendererBuilder;
use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl TypstRendererBuilder {
    pub async fn add_web_fonts(&mut self, font: js_sys::Array) -> ZResult<()> {
        self.searcher.add_web_fonts(font).await
    }
}
