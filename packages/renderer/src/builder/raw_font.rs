use crate::TypstRendererBuilder;
use js_sys::Uint8Array;
use typst::util::Buffer;
use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl TypstRendererBuilder {
    pub async fn add_raw_font(&mut self, font_buffer: Uint8Array) -> ZResult<()> {
        self.add_raw_font_internal(font_buffer.to_vec().into());
        Ok(())
    }
}

impl TypstRendererBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Buffer) {
        self.searcher.add_font_data(font_buffer);
    }
}
