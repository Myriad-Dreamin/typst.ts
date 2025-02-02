use reflexo_typst::error::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{TypstRenderer, TypstRendererBuilder};

#[wasm_bindgen]
impl TypstRendererBuilder {
    pub async fn add_glyph_pack(&mut self, pack: JsValue) -> Result<()> {
        let pack = serde_wasm_bindgen::from_value(pack).unwrap();
        self.searcher.add_glyph_pack(pack).await
    }
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn load_glyph_pack(&self, v: JsValue) -> Result<()> {
        let mut font_resolver = self.session_mgr.font_resolver.write().unwrap();
        font_resolver.add_glyph_packs(
            serde_wasm_bindgen::from_value(v).map_err(map_string_err("GlyphBundleFmt"))?,
        );
        Ok(())
    }
}
