use std::sync::Arc;

use js_sys::{JsString, Uint8Array};
use typst_ts_compiler::font::web::BrowserFontSearcher;
pub use typst_ts_compiler::*;
use typst_ts_core::{cache::FontInfoCache, Exporter};
use wasm_bindgen::prelude::*;

use crate::utils::console_log;

pub mod builder;

pub(crate) mod utils;

#[wasm_bindgen]
pub struct TypstCompiler {
    pub(crate) world: TypstBrowserWorld,
}

impl TypstCompiler {
    pub async fn new(searcher: BrowserFontSearcher) -> Result<Self, JsValue> {
        Ok(Self {
            world: TypstBrowserWorld::new_raw(
                std::path::Path::new("/").to_owned(),
                searcher.into(),
            ),
        })
    }
}

#[wasm_bindgen]
pub fn get_font_info(buffer: Uint8Array) -> JsValue {
    serde_wasm_bindgen::to_value(&FontInfoCache::from_data(buffer.to_vec().as_slice())).unwrap()
}

#[wasm_bindgen]
impl TypstCompiler {
    pub fn reset(&mut self) {
        // reset the world caches
        self.world.reset();
    }

    pub fn add_source(&mut self, path: String, content: String, is_main: bool) -> bool {
        // checkout the entry file
        match self
            .world
            .resolve_with(std::path::Path::new(&path), &content)
        {
            Ok(id) => {
                if is_main {
                    self.world.main = id;
                    console_log!("main: {:?}", id);
                }
                true
            }
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn modify_font_data(&mut self, idx: usize, buffer: Uint8Array) {
        self.world
            .font_resolver
            .modify_font_data(idx, buffer.to_vec().into());
    }

    pub fn rebuild(&mut self) {
        if self.world.font_resolver.partial_resolved() {
            self.world.font_resolver.rebuild();
        }
    }

    pub fn get_loaded_fonts(&mut self) -> Vec<JsString> {
        self.world
            .font_resolver
            .loaded_fonts()
            .map(|s| format!("<{}, {:?}>", s.0, s.1).into())
            .collect()
    }

    pub fn get_ast(&mut self) -> Result<String, JsValue> {
        let ast_exporter = typst_ts_core::exporter_builtins::VecExporter::new(
            typst_ts_ast_exporter::AstExporter::default(),
        );

        // compile and export document
        let doc = typst::compile(&self.world).unwrap();
        let data = ast_exporter.export(&self.world, Arc::new(doc)).unwrap();

        let converted =
            ansi_to_html::convert_escaped(String::from_utf8(data).unwrap().as_str()).unwrap();
        Ok(converted)
    }

    pub fn get_artifact(&mut self, _format: String) -> Result<Vec<u8>, JsValue> {
        let ir_exporter = typst_ts_core::exporter_builtins::VecExporter::new(
            typst_ts_tir_exporter::IRArtifactExporter::default(),
        );

        let doc = typst::compile(&self.world).unwrap();
        let artifact_bytes = ir_exporter
            .export(&self.world, Arc::new((&doc).into()))
            .unwrap();
        Ok(artifact_bytes)
    }
}
