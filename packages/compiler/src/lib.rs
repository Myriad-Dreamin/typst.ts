use std::sync::{Arc, Mutex};

use typst_ts_compiler::font::web::BrowserFontSearcher;
pub use typst_ts_compiler::*;
use typst_ts_core::DocumentExporter;
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

    pub fn get_ast(&mut self) -> Result<String, JsValue> {
        let data = Arc::new(Mutex::new(vec![]));

        let inner_data = data.clone();
        let ast_exporter = typst_ts_ast_exporter::AstExporter::new_vec(Box::new(move |v| {
            let mut data = inner_data.lock().unwrap();
            *data = v;
            Ok(())
        }));

        // compile and export document
        typst::compile(&self.world)
            .and_then(|output| ast_exporter.export(&self.world, &output))
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
