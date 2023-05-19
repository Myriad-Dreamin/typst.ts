use std::sync::{Arc, Mutex};

use js_sys::{JsString, Uint8Array};
use typst_ts_compiler::font::web::BrowserFontSearcher;
pub use typst_ts_compiler::*;
use typst_ts_core::{cache::FontInfoCache, DocumentExporter};
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
        let data = Arc::new(Mutex::new(vec![]));

        let inner_data = data.clone();
        let ast_exporter = typst_ts_ast_exporter::AstExporter::new_vec(Box::new(move |v| {
            let mut data = inner_data.lock().unwrap();
            *data = v;
            Ok(())
        }));

        // let artifact = Arc::new(Mutex::new(None));

        // compile and export document
        typst::compile(&self.world)
            .and_then(|output| {
                // let mut artifact = artifact.lock().unwrap();
                // artifact = Some(Artifact::from(&output));
                // drop(artifact);

                ast_exporter.export(&self.world, Arc::new(output))
            })
            .unwrap();
        let converted = ansi_to_html::convert_escaped(
            String::from_utf8(data.lock().unwrap().clone())
                .unwrap()
                .as_str(),
        )
        .unwrap();
        Ok(converted)
    }
}
