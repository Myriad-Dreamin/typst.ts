use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use typst_ts_core::{error::prelude::*, Bytes};
use typst_ts_core::{
    font::web::BrowserFontSearcher,
    package::browser::{ProxyContext, ProxyRegistry},
    vfs::browser::ProxyAccessModel,
};

use crate::TypstCompiler;

#[wasm_bindgen]
pub struct TypstCompilerBuilder {
    access_model: Option<ProxyAccessModel>,
    package_registry: Option<ProxyRegistry>,
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstCompilerBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ZResult<TypstCompilerBuilder> {
        console_error_panic_hook::set_once();
        let mut res = Self {
            access_model: None,
            package_registry: None,
            searcher: BrowserFontSearcher::new(),
        };
        res.set_dummy_access_model()?;
        Ok(res)
    }

    pub fn set_dummy_access_model(&mut self) -> ZResult<()> {
        self.access_model = Some(ProxyAccessModel {
            context: wasm_bindgen::JsValue::UNDEFINED,
            mtime_fn: js_sys::Function::new_no_args("return 0"),
            is_file_fn: js_sys::Function::new_no_args("return true"),
            real_path_fn: js_sys::Function::new_with_args("path", "return path"),
            read_all_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy AccessModel, please initialize compiler with withAccessModel()')",
            ),
        });
        self.package_registry = Some(ProxyRegistry {
            context: ProxyContext::new(wasm_bindgen::JsValue::UNDEFINED),
            real_resolve_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy Registry, please initialize compiler with withPackageRegistry()')",
            ),
        });
        Ok(())
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

    pub async fn set_package_registry(
        &mut self,
        context: JsValue,
        real_resolve_fn: js_sys::Function,
    ) -> ZResult<()> {
        self.package_registry = Some(ProxyRegistry {
            context: ProxyContext::new(context),
            real_resolve_fn,
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
        let registry = self.package_registry.ok_or_else(|| {
            "TypstCompilerBuilder::build: package_registry is not set".to_string()
        })?;
        TypstCompiler::new(access_model, registry, self.searcher).await
    }
}

impl TypstCompilerBuilder {
    pub fn add_raw_font_internal(&mut self, font_buffer: Bytes) {
        self.searcher.add_font_data(font_buffer);
    }
}
