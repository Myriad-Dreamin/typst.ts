use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use reflexo_typst::font::web::BrowserFontSearcher;
use reflexo_typst::package::registry::{JsRegistry, ProxyContext};
use reflexo_typst::vfs::browser::ProxyAccessModel;
use reflexo_typst::{error::prelude::*, Bytes as TypstBytes};

use crate::TypstCompiler;

#[wasm_bindgen]
pub struct TypstCompilerBuilder {
    access_model: Option<ProxyAccessModel>,
    package_registry: Option<JsRegistry>,
    searcher: BrowserFontSearcher,
}

#[wasm_bindgen]
impl TypstCompilerBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstCompilerBuilder> {
        console_error_panic_hook::set_once();
        let mut res = Self {
            access_model: None,
            package_registry: None,
            searcher: BrowserFontSearcher::new(),
        };
        res.set_dummy_access_model()?;
        Ok(res)
    }

    pub fn set_dummy_access_model(&mut self) -> Result<()> {
        self.access_model = Some(ProxyAccessModel {
            context: wasm_bindgen::JsValue::UNDEFINED,
            mtime_fn: js_sys::Function::new_no_args("return 0"),
            is_file_fn: js_sys::Function::new_no_args("return true"),
            real_path_fn: js_sys::Function::new_with_args("path", "return path"),
            read_all_fn: js_sys::Function::new_no_args(
                "throw new Error('Dummy AccessModel, please initialize compiler with withAccessModel()')",
            ),
        });
        self.package_registry = Some(JsRegistry {
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
    ) -> Result<()> {
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
    ) -> Result<()> {
        self.package_registry = Some(JsRegistry {
            context: ProxyContext::new(context),
            real_resolve_fn,
        });

        Ok(())
    }

    // 400 KB
    pub async fn add_raw_font(&mut self, data: Uint8Array) -> Result<()> {
        self.searcher.add_font_data(TypstBytes::new(data.to_vec()));
        Ok(())
    }

    // 100 KB
    pub async fn add_web_fonts(&mut self, fonts: js_sys::Array) -> Result<()> {
        self.searcher.add_web_fonts(fonts).await
    }

    pub async fn build(self) -> Result<TypstCompiler, JsValue> {
        let access_model = self
            .access_model
            .ok_or_else(|| "TypstCompilerBuilder::build: access_model is not set".to_string())?;
        let registry = self.package_registry.ok_or_else(|| {
            "TypstCompilerBuilder::build: package_registry is not set".to_string()
        })?;

        let searcher = self.searcher;
        #[cfg(feature = "fonts")]
        let mut searcher = searcher;
        #[cfg(feature = "fonts")]
        searcher.add_embedded();

        TypstCompiler::new(access_model, registry, searcher).await
    }
}
