use std::{path::Path, sync::Arc};

use js_sys::{JsString, Uint8Array};
use typst::font::Font;
pub use typst_ts_compiler::*;
use typst_ts_compiler::{
    font::web::BrowserFontSearcher,
    service::{CompileDriverImpl, Compiler},
    vfs::browser::ProxyAccessModel,
};
use typst_ts_core::{cache::FontInfoCache, error::prelude::*, Exporter, FontLoader};
use wasm_bindgen::prelude::*;

use crate::utils::console_log;

pub mod builder;

pub(crate) mod utils;

#[wasm_bindgen]
pub struct TypstCompiler {
    pub(crate) compiler: CompileDriverImpl<TypstBrowserWorld>,
}

impl TypstCompiler {
    pub async fn new(
        access_model: ProxyAccessModel,
        searcher: BrowserFontSearcher,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            compiler: CompileDriverImpl::new(TypstBrowserWorld::new(
                std::path::Path::new("/").to_owned(),
                access_model,
                searcher.into(),
            )),
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
        self.compiler.reset().unwrap();
    }

    pub fn add_source(&mut self, path: String, content: String, is_main: bool) -> bool {
        let path = Path::new(&path).to_owned();
        match self.compiler.map_shadow(&path, &content) {
            Ok(_) => {
                if is_main {
                    self.compiler.set_entry_file(path);
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
        _snapshot: JsValue,
        _font_cb: js_sys::Function,
    ) -> Result<DocumentReference, JsValue> {
        todo!()
        // let mut snapshot: WorldSnapshot =
        // serde_wasm_bindgen::from_value(snapshot).unwrap();
        // if let Some(font_profile) = snapshot.font_profile.take() {
        //     for item in font_profile.items {
        //         let path = if let Some(path) = item.path() {
        //             path.clone()
        //         } else {
        //             continue;
        //         };
        //         // item.info
        //         for (idx, info) in item.info.into_iter().enumerate() {
        //             let font_idx = info.index().unwrap_or(idx as u32);
        //             self.compiler.world_mut().font_resolver.append_font(
        //                 info.info,
        //                 FontSlot::new_boxed(SnapshotFontLoader {
        //                     font_cb: font_cb.clone(),
        //                     index: font_idx,
        //                     path: path.clone(),
        //                 }),
        //             );
        //         }
        //     }
        // };
        // self.rebuild();

        // let artifact_header = snapshot.artifact_header;
        // let cap = base64::engine::general_purpose::STANDARD
        //     .internal_decoded_len_estimate(snapshot.artifact_data.len())
        //     .decoded_len_estimate();
        // Ok(DocumentReference {
        //     doc: Some(Arc::new(
        //         artifact_ir::Artifact::with_initializer(
        //             cap,
        //             |buf_mut| {
        //                 base64::engine::general_purpose::STANDARD
        //                     .decode_slice(snapshot.artifact_data.as_bytes(),
        // buf_mut)                     .unwrap();
        //             },
        //             artifact_header,
        //         )
        //         .to_document(&self.compiler.world_mut().font_resolver),
        //     )),
        // })
    }

    pub fn modify_font_data(&mut self, idx: usize, buffer: Uint8Array) {
        self.compiler
            .world_mut()
            .font_resolver
            .modify_font_data(idx, buffer.to_vec().into());
    }

    pub fn rebuild(&mut self) {
        if self.compiler.world_mut().font_resolver.partial_resolved() {
            self.compiler.world_mut().font_resolver.rebuild();
        }
    }

    pub fn get_loaded_fonts(&mut self) -> Vec<JsString> {
        self.compiler
            .world_mut()
            .font_resolver
            .loaded_fonts()
            .map(|s| format!("<{}, {:?}>", s.0, s.1).into())
            .collect()
    }

    pub fn get_ast(&mut self, main_file_path: String) -> Result<String, JsValue> {
        self.compiler
            .set_entry_file(Path::new(&main_file_path).to_owned());

        let ast_exporter = typst_ts_core::exporter_builtins::VecExporter::new(
            typst_ts_ast_exporter::AstExporter::default(),
        );

        // compile and export document
        let doc = self.compiler.compile().unwrap();
        let data = ast_exporter
            .export(self.compiler.world(), Arc::new(doc))
            .unwrap();

        let converted =
            ansi_to_html::convert_escaped(String::from_utf8(data).unwrap().as_str()).unwrap();
        Ok(converted)
    }

    pub fn get_artifact(&mut self, _format: String) -> Result<Vec<u8>, JsValue> {
        let ir_exporter = typst_ts_core::exporter_builtins::VecExporter::new(
            typst_ts_svg_exporter::SvgModuleExporter::default(),
        );

        let doc = self.compiler.compile().unwrap();
        let artifact_bytes = ir_exporter
            .export(self.compiler.world(), Arc::new(doc))
            .unwrap();
        Ok(artifact_bytes)
    }

    pub fn compile(&mut self, _main_file_path: String) -> Result<DocumentReference, JsValue> {
        self.compiler
            .set_entry_file(Path::new(&_main_file_path).to_owned());

        let doc = self.compiler.compile().unwrap();
        Ok(DocumentReference {
            doc: Some(Arc::new(doc)),
        })
    }

    // todo: move to renderer
    pub async fn render_page_to_canvas(
        &mut self,
        _canvas: &web_sys::CanvasRenderingContext2d,
        _doc: &DocumentReference,
        _page_off: usize,
        _pixel_per_pt: f32,
        _background_color: String,
    ) -> ZResult<JsValue> {
        // let doc = doc.doc_ref();
        // let mut worker = CanvasRenderTask::<DefaultRenderFeature>::new(
        //     canvas,
        //     doc,
        //     page_off,
        //     pixel_per_pt,
        //     Color::Rgba(
        //         RgbaColor::from_str(&background_color)
        //             .map_err(map_err("Renderer.InvalidBackgroundColor"))?,
        //     ),
        // )?;

        // worker.render(&doc.pages[page_off]).await?;
        // serde_wasm_bindgen::to_value(&worker.text_content)
        //     .map_err(map_into_err::<JsValue, _>("Compiler.EncodeContent"))
        todo!()
    }
}
