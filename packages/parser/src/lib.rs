use std::{path::Path, sync::Arc};

use js_sys::Uint32Array;
use reflexo_typst::error::prelude::*;
use reflexo_typst::parser::{
    get_semantic_tokens_full, get_semantic_tokens_legend, OffsetEncoding, SemanticToken,
};
use typst::syntax::{FileId, VirtualPath};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct TypstParserBuilder {}

#[wasm_bindgen]
impl TypstParserBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<TypstParserBuilder> {
        console_error_panic_hook::set_once();
        Ok(Self {})
    }

    pub async fn build(self) -> Result<TypstParser, JsValue> {
        Ok(TypstParser {})
    }
}

#[wasm_bindgen]
pub struct TypstParser {}

impl TypstParser {
    pub fn get_semantic_tokens_inner(
        &self,
        src: String,
        offset_encoding: OffsetEncoding,
    ) -> Arc<Vec<SemanticToken>> {
        let src = typst::syntax::Source::new(
            FileId::new(None, VirtualPath::new(Path::new("/main.typ"))),
            src,
        );

        Arc::new(get_semantic_tokens_full(&src, offset_encoding))
    }
}

#[wasm_bindgen]
impl TypstParser {
    pub fn get_semantic_token_legend(&self) -> Result<JsValue, JsValue> {
        let legend = get_semantic_tokens_legend();
        serde_wasm_bindgen::to_value(&legend).map_err(|e| format!("{e:?}").into())
    }

    pub fn get_semantic_tokens_by_string(
        &self,
        src: String,
        offset_encoding: String,
    ) -> Result<Uint32Array> {
        let tokens = self.get_semantic_tokens_inner(src, match offset_encoding.as_str() {
            "utf-16" => OffsetEncoding::Utf16,
           "utf-8" => OffsetEncoding::Utf8,
             _ => {
                 return Err(error_once!("Unsupported offset encoding", offset_encoding: offset_encoding));
             }
         });
        let mut result = Vec::new();
        for token in tokens.iter() {
            result.push(token.delta_line);
            result.push(token.delta_start_character);
            result.push(token.length);
            result.push(token.token_type);
            result.push(token.token_modifiers);
        }

        Ok(Uint32Array::from(&result[..]))
    }
}
