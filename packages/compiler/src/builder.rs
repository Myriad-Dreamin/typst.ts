use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use typst_ts_compiler::{font::web::BrowserFontSearcher, vfs::browser::ProxyAccessModel};
use typst_ts_core::{error::prelude::*, Bytes};

use crate::TypstCompiler;

#[wasm_bindgen]
pub struct TypstCompilerBuilder {
    access_model: Option<ProxyAccessModel>,
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstCompilerBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ZResult<TypstCompilerBuilder> {
        console_error_panic_hook::set_once();
        Ok(Self {
            access_model: None,
            searcher: BrowserFontSearcher::new(),
        })
    }

    pub async fn set_access_model(
        &mut self,
        context: JsValue,
        mtime_fn: js_sys::Function,
        is_file_fn: js_sys::Function,
        real_path_fn: js_sys::Function,
        read_all_fn: js_sys::Function,
    ) -> ZResult<()> {
        self.access_model = Some(ProxyAccessModel {
            context,
            mtime_fn,
            is_file_fn,
            real_path_fn,
            read_all_fn,
        });

        Ok(())
    }

    // 400 KB
    pub async fn add_raw_font(&mut self, font_buffer: Uint8Array) -> ZResult<()> {
        self.add_raw_font_internal(font_buffer.to_vec().into());
        Ok(())
    }

    // 100 KB
    pub async fn add_web_fonts(&mut self, fonts: js_sys::Array) -> ZResult<()> {
        self.searcher.add_web_fonts(fonts).await
    }

    pub async fn add_glyph_pack(&mut self, _pack: JsValue) -> ZResult<()> {
        self.searcher.add_glyph_pack().await
    }

    pub async fn build(self) -> Result<TypstCompiler, JsValue> {
        let access_model = self
            .access_model
            .ok_or_else(|| "TypstCompilerBuilder::build: access_model is not set".to_string())?;
        TypstCompiler::new(access_model, self.searcher).await
    }
}

impl TypstCompilerBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Bytes) {
        self.searcher.add_font_data(font_buffer);
    }
}
