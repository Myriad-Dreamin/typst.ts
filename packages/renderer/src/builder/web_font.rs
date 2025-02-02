use reflexo_typst::error::prelude::*;
use wasm_bindgen::prelude::*;

use crate::TypstRendererBuilder;

#[wasm_bindgen]
impl TypstRendererBuilder {
    pub async fn add_web_fonts(&mut self, font: js_sys::Array) -> Result<()> {
        self.searcher.add_web_fonts(font).await
    }
}
