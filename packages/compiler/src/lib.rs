use std::{path::Path, sync::Arc};

use base64::Engine;
use js_sys::{JsString, Uint8Array};
use typst::font::Font;
pub use typst_ts_compiler::*;
use typst_ts_compiler::{
    font::web::BrowserFontSearcher,
    service::{CompileDriverImpl, Compiler},
    vfs::browser::ProxyAccessModel,
    world::WorldSnapshot,
};
use typst_ts_core::{cache::FontInfoCache, error::prelude::*, Exporter, FontLoader, FontSlot};
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

    pub fn add_source(&mut self, path: &str, content: &str, is_main: bool) -> bool {
        let path = Path::new(path).to_owned();
        match self.compiler.map_shadow(&path, content) {
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
        snapshot: JsValue,
        font_cb: js_sys::Function,
    ) -> Result<Vec<u8>, JsValue> {
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
                    self.compiler.world_mut().font_resolver.append_font(
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

        let artifact = base64::engine::general_purpose::STANDARD
            .decode(snapshot.artifact_data)
            .unwrap();
        Ok(artifact)
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

    pub fn get_artifact(&mut self, fmt: String) -> Result<Vec<u8>, JsValue> {
        if fmt != "vector" {
            return Err(error_once!("Unsupported fmt", format: fmt).into());
        }

        let ir_exporter = typst_ts_core::exporter_builtins::VecExporter::new(
            typst_ts_svg_exporter::SvgModuleExporter::default(),
        );

        let doc = self.compiler.compile().unwrap();
        let artifact_bytes = ir_exporter
            .export(self.compiler.world(), Arc::new(doc))
            .unwrap();
        Ok(artifact_bytes)
    }

    pub fn compile(&mut self, main_file_path: String) -> Result<Vec<u8>, JsValue> {
        self.compiler
            .set_entry_file(Path::new(&main_file_path).to_owned());

        self.get_artifact("vector".into())
    }
}
