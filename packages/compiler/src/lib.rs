use std::{str::FromStr, sync::Arc};

use base64::Engine;
use js_sys::{JsString, Uint8Array};
use typst::{
    font::Font,
    geom::{Color, RgbaColor},
    World,
};
pub use typst_ts_compiler::*;
use typst_ts_compiler::{
    font::web::BrowserFontSearcher, vfs::browser::ProxyAccessModel, world::WorldSnapshot,
};
use typst_ts_core::{
    artifact_ir, cache::FontInfoCache, error::prelude::*, Exporter, FontLoader, FontSlot,
};
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
    doc: Option<Arc<typst::doc::Document>>,
}

impl DocumentReference {
    pub fn doc_ref(&self) -> &typst::doc::Document {
        self.doc.as_ref().unwrap()
    }
}

#[wasm_bindgen]
impl DocumentReference {
    pub fn page_total(&self) -> usize {
        self.doc_ref().pages.len()
    }

    // width, height
    pub fn page_width(&self) -> f64 {
        if let Some(page) = self.doc_ref().pages.first() {
            page.size().x.to_pt()
        } else {
            0.0
        }
    }

    pub fn page_height(&self) -> f64 {
        if let Some(page) = self.doc_ref().pages.first() {
            page.size().y.to_pt()
        } else {
            0.0
        }
    }
}

struct SnapshotFontLoader {
    font_cb: js_sys::Function,
    index: u32,
    path: String,
}

impl FontLoader for SnapshotFontLoader {
    fn load(&mut self) -> Option<Font> {
        let buf = self
            .font_cb
            .call1(&self.font_cb, &self.path.clone().into())
            .unwrap();
        let buf = buf.dyn_ref::<Uint8Array>()?;
        let buf = buf.to_vec();
        Font::new(buf.into(), self.index)
    }
}

// todo: remove this
unsafe impl Send for SnapshotFontLoader {}

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

    pub fn load_snapshot(
        &mut self,
        snapshot: JsValue,
        font_cb: js_sys::Function,
    ) -> Result<DocumentReference, JsValue> {
        let mut snapshot: WorldSnapshot = serde_wasm_bindgen::from_value(snapshot).unwrap();
        if let Some(font_profile) = snapshot.font_profile.take() {
            for item in font_profile.items {
                let path = if let Some(path) = item.path() {
                    path.clone()
                } else {
                    continue;
                };
                // item.info
                for (idx, info) in item.info.into_iter().enumerate() {
                    let font_idx = info.index().unwrap_or(idx as u32);
                    self.world.font_resolver.append_font(
                        info.info,
                        FontSlot::new_boxed(SnapshotFontLoader {
                            font_cb: font_cb.clone(),
                            index: font_idx,
                            path: path.clone(),
                        }),
                    );
                }
            }
        };
        self.rebuild();

        let artifact_header = snapshot.artifact_header;
        let artifact_data = base64::engine::general_purpose::STANDARD
            .decode(snapshot.artifact_data)
            .unwrap();

        Ok(DocumentReference {
            doc: Some(Arc::new(
                artifact_ir::Artifact {
                    metadata: artifact_header.metadata,
                    pages: artifact_header.pages,
                    buffer: artifact_data,
                }
                .to_document(&self.world.font_resolver),
            )),
        })
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

    pub fn compile(&mut self, main_file_path: String) -> Result<DocumentReference, JsValue> {
        self.world.main = self
            .world
            .resolve(std::path::Path::new(&main_file_path))
            .unwrap();

        let doc = typst::compile(&self.world).unwrap();
        Ok(DocumentReference {
            doc: Some(Arc::new(doc)),
        })
    }

    // todo: move to renderer
    pub fn render_page_to_canvas(
        &mut self,
        canvas: &web_sys::CanvasRenderingContext2d,
        doc: &DocumentReference,
        page_off: usize,
        pixel_per_pt: f32,
        background_color: String,
    ) -> ZResult<JsValue> {
        let doc = doc.doc_ref();
        let mut worker = typst_ts_canvas_exporter::CanvasRenderTask::new(
            canvas,
            doc,
            page_off,
            pixel_per_pt,
            Color::Rgba(
                RgbaColor::from_str(&background_color)
                    .map_err(map_err("Renderer.InvalidBackgroundColor"))?,
            ),
        )?;

        worker.render(&doc.pages[page_off])?;
        serde_wasm_bindgen::to_value(&worker.content)
            .map_err(map_into_err::<JsValue, _>("Compiler.EncodeContent"))
    }
}
