use std::{str::FromStr, sync::Arc};

use js_sys::{JsString, Uint8Array};
use typst::{
    geom::{Color, RgbaColor},
    World,
};
use typst_ts_canvas_exporter::LigatureMap;
pub use typst_ts_compiler::*;
use typst_ts_compiler::{font::web::BrowserFontSearcher, vfs::browser::ProxyAccessModel};
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
    pub async fn new(
        access_model: ProxyAccessModel,
        searcher: BrowserFontSearcher,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            world: TypstBrowserWorld::new(
                std::path::Path::new("/").to_owned(),
                access_model,
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
pub struct DocumentReference {
    doc: Arc<typst::doc::Document>,
}

#[wasm_bindgen]
impl DocumentReference {
    pub fn page_total(&self) -> usize {
        self.doc.pages.len()
    }

    // width, height
    pub fn page_width(&self) -> f64 {
        if let Some(page) = self.doc.pages.first() {
            page.size().x.to_pt()
        } else {
            0.0
        }
    }

    pub fn page_height(&self) -> f64 {
        if let Some(page) = self.doc.pages.first() {
            page.size().y.to_pt()
        } else {
            0.0
        }
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

    pub fn get_ast(&mut self, main_file_path: String) -> Result<String, JsValue> {
        self.world.main = self
            .world
            .resolve(std::path::Path::new(&main_file_path))
            .unwrap();

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

    // todo: move to renderer
    pub fn render_page_to_canvas(
        &mut self,
        canvas: &web_sys::CanvasRenderingContext2d,
        doc: &DocumentReference,
        page_off: usize,
        pixel_per_pt: f32,
        background_color: String,
    ) -> Result<JsValue, JsValue> {
        let d = LigatureMap::default();
        let mut worker = typst_ts_canvas_exporter::CanvasRenderTask::new(
            canvas,
            &doc.doc,
            &d,
            page_off,
            pixel_per_pt,
            Color::Rgba(RgbaColor::from_str(&background_color)?),
        );

        worker.render(&doc.doc.pages[page_off]);
        Ok(serde_wasm_bindgen::to_value(&worker.content).unwrap())
    }
}
