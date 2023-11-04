use std::path::Path;

use base64::Engine;
use js_sys::{JsString, Uint32Array, Uint8Array};
use typst::{eval::IntoValue, font::Font};
pub use typst_ts_compiler::*;
use typst_ts_compiler::{
    font::web::BrowserFontSearcher,
    package::browser::ProxyRegistry,
    parser::OffsetEncoding,
    service::{CompileDriverImpl, Compiler},
    vfs::browser::ProxyAccessModel,
    world::WorldSnapshot,
};
use typst_ts_core::{
    cache::FontInfoCache, error::prelude::*, DynExporter, Exporter, FontLoader, FontSlot,
    TypstDocument,
};
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
        registry: ProxyRegistry,
        searcher: BrowserFontSearcher,
    ) -> Result<Self, JsValue> {
        Ok(Self {
            compiler: CompileDriverImpl::new(TypstBrowserWorld::new(
                std::path::Path::new("/").to_owned(),
                access_model,
                registry,
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

// todo: design error handling
// todo: we return a string for now which is better than nothing
#[wasm_bindgen]
impl TypstCompiler {
    pub fn reset(&mut self) -> Result<(), JsValue> {
        // reset the world caches
        self.compiler.reset().map_err(|e| format!("{e:?}"))?;

        Ok(())
    }

    pub fn add_source(&mut self, path: &str, content: &str) -> bool {
        let path = Path::new(path).to_owned();
        match self.compiler.map_shadow(&path, content.as_bytes().into()) {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn map_shadow(&mut self, path: &str, content: &[u8]) -> bool {
        let path = Path::new(path).to_owned();
        match self.compiler.map_shadow(&path, content.into()) {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn unmap_shadow(&mut self, path: &str) -> bool {
        let path = Path::new(path).to_owned();
        match self.compiler.unmap_shadow(&path) {
            Ok(_) => true,
            Err(e) => {
                console_log!("Error: {:?}", e);
                false
            }
        }
    }

    pub fn reset_shadow(&mut self) {
        self.compiler.reset_shadow()
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
        let doc = self.compiler.compile().map_err(|e| format!("{e:?}"))?;
        let data = ast_exporter
            .export(self.compiler.world(), doc)
            .map_err(|e| format!("{e:?}"))?;

        let converted = ansi_to_html::convert_escaped(
            String::from_utf8(data)
                .map_err(|e| format!("{e:?}"))?
                .as_str(),
        )
        .map_err(|e| format!("{e:?}"))?;
        Ok(converted)
    }

    pub fn get_semantic_token_legend(&mut self) -> Result<JsValue, JsValue> {
        let tokens = self.compiler.world_mut().get_semantic_token_legend();
        serde_wasm_bindgen::to_value(tokens.as_ref()).map_err(|e| format!("{e:?}").into())
    }

    pub fn get_semantic_tokens(
        &mut self,
        offset_encoding: String,
        file_path: Option<String>,
        result_id: Option<String>,
    ) -> Result<js_sys::Object, JsValue> {
        if let Some(result_id) = result_id {
            return Err(
                error_once!("Not implemented", result_id: format!("{:?}", result_id)).into(),
            );
        }

        let tokens = self.compiler.world_mut().get_semantic_tokens(
            file_path,
            match offset_encoding.as_str() {
               "utf-16" => OffsetEncoding::Utf16,
              "utf-8" => OffsetEncoding::Utf8,
                _ => {
                    return Err(error_once!("Unsupported offset encoding", offset_encoding: offset_encoding).into());
                }
            },
        );
        let mut result = Vec::new();
        for token in tokens.iter() {
            result.push(token.delta_line);
            result.push(token.delta_start_character);
            result.push(token.length);
            result.push(token.token_type);
            result.push(token.token_modifiers);
        }

        let semantic_tokens = js_sys::Object::new();
        js_sys::Reflect::set(
            &semantic_tokens,
            &"data".into(),
            &Uint32Array::from(&result[..]).into(),
        )?;
        js_sys::Reflect::set(
            &semantic_tokens,
            &"resultId".into(),
            &JsString::from("").into(),
        )?;

        Ok(semantic_tokens)
    }

    pub fn get_artifact(&mut self, fmt: String) -> Result<Vec<u8>, JsValue> {
        let vec_exporter: DynExporter<TypstDocument, Vec<u8>> = match fmt.as_str() {
            "vector" => Box::new(typst_ts_core::exporter_builtins::VecExporter::new(
                typst_ts_svg_exporter::SvgModuleExporter::default(),
            )),
            "pdf" => Box::<typst_ts_pdf_exporter::PdfDocExporter>::default(),
            _ => {
                return Err(error_once!("Unsupported fmt", format: fmt).into());
            }
        };

        let doc = self.compiler.compile().map_err(|e| format!("{e:?}"))?;
        let artifact_bytes = vec_exporter
            .export(self.compiler.world(), doc)
            .map_err(|e| format!("{e:?}"))?;
        Ok(artifact_bytes)
    }

    pub fn query(
        &mut self,
        main_file_path: String,
        selector: String,
        field: Option<String>,
    ) -> Result<String, JsValue> {
        self.compiler
            .set_entry_file(Path::new(&main_file_path).to_owned());

        let doc = self.compiler.compile().map_err(|e| format!("{e:?}"))?;
        let elements: Vec<typst::model::Content> = self
            .compiler
            .query(selector, &doc)
            .map_err(|e| format!("{e:?}"))?;

        let mapped: Vec<_> = elements
            .into_iter()
            .filter_map(|c| match &field {
                Some(field) => c.field(field),
                _ => Some(c.into_value()),
            })
            .collect();

        Ok(serde_json::to_string_pretty(&mapped).map_err(|e| format!("{e:?}"))?)
    }

    pub fn compile(&mut self, main_file_path: String, fmt: String) -> Result<Vec<u8>, JsValue> {
        self.compiler
            .set_entry_file(Path::new(&main_file_path).to_owned());

        self.get_artifact(fmt)
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    #![allow(clippy::await_holding_lock)]

    use sha2::Digest;
    use typst_ts_svg_exporter::MultiSvgDocument;
    use typst_ts_test_common::web_artifact::get_corpus;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    use crate::builder::TypstCompilerBuilder;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    async fn get_source(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.typ", name)).await.unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    async fn get_ir_artifact(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.artifact.sir.in", name))
            .await
            .unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    fn render_svg(artifact: &[u8]) -> String {
        let doc = MultiSvgDocument::from_slice(artifact);
        type UsingExporter =
            typst_ts_svg_exporter::SvgExporter<typst_ts_svg_exporter::SvgExportFeature>;

        let node = doc.layouts[0].unwrap_single();
        let view = node.pages(&doc.module).unwrap();
        UsingExporter::render_flat_svg(&doc.module, view.pages())
    }

    async fn render_test_template(point: &str, source: &[u8], artifact: &[u8]) {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let mut compiler = TypstCompilerBuilder::new().unwrap();
        compiler.set_dummy_access_model().await.unwrap();
        let mut compiler = compiler.build().await.unwrap();
        let start = performance.now();
        if !compiler.add_source(
            &format!("/{point}.typ"),
            std::str::from_utf8(source).unwrap(),
            true,
        ) {
            panic!("Failed to add source {point}");
        }
        let end = performance.now();
        let time_used = end - start;

        let browser_artifact = compiler.compile(format!("/{point}.typ")).unwrap();

        let x_svg = render_svg(&browser_artifact);
        let y_svg = render_svg(artifact);

        let x_hash = hash_bytes(&x_svg);
        let y_hash = hash_bytes(&y_svg);

        use base64::Engine;
        let e = base64::engine::general_purpose::STANDARD;
        let x = web_sys::HtmlImageElement::new().unwrap();
        x.set_src(&format!("data:image/svg+xml;base64,{}", e.encode(x_svg)));
        x.set_attribute("style", "flex: 1;").unwrap();
        let y = web_sys::HtmlImageElement::new().unwrap();
        y.set_src(&format!("data:image/svg+xml;base64,{}", e.encode(y_svg)));
        y.set_attribute("style", "flex: 1;").unwrap();

        let div = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        div.set_attribute("style", "display block; border: 1px solid #000;")
            .unwrap();

        let title = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        title.set_inner_html(&format!(
            "{point} => {time_used:.3}ms, hash_cmp: {x_hash} v.s. {y_hash}",
        ));

        div.append_child(&title).unwrap();

        let cmp = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        cmp.set_attribute("style", "display: flex;").unwrap();

        cmp.append_child(&x).unwrap();
        cmp.append_child(&y).unwrap();

        div.append_child(&cmp).unwrap();

        let body = window.document().unwrap().body().unwrap();

        body.append_child(&div).unwrap();
    }

    async fn render_test_from_corpus(path: &str) {
        let point = path.replace('/', "_");
        let ir_point = format!("{}_artifact_ir", point);

        render_test_template(
            &ir_point,
            &get_source(path).await,
            &get_ir_artifact(path).await,
        )
        .await;
    }

    macro_rules! make_test_point {
        ($name:ident, $($path:literal),+ $(,)?) => {
            #[wasm_bindgen_test]
            async fn $name() {
                $(
                    render_test_from_corpus($path).await;
                )*
            }
        };
    }

    make_test_point!(test_render_math_main, "math/main");
    make_test_point!(test_render_math_undergradmath, "math/undergradmath");
}
