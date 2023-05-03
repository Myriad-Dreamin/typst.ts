use super::artifact::{convert_pair, ArtifactJsBuilder};
use crate::utils::console_log;
use typst_ts_core::artifact_ir::ArtifactHeader;
use typst_ts_core::artifact_ir::{core::ItemArray, doc::Frame};
use wasm_bindgen::prelude::*;

pub struct IRArtifactHeaderJsBuilder {
    builder: ArtifactJsBuilder,
}

impl IRArtifactHeaderJsBuilder {
    pub fn new() -> Self {
        Self {
            builder: ArtifactJsBuilder {},
        }
    }

    pub fn from_value(&mut self, val: JsValue) -> Result<ArtifactHeader, JsValue> {
        let mut metadata = ArtifactHeader::default();

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or("typst: not a js object")?)
            .iter()
            .map(convert_pair)
        {
            let k = k.as_string().ok_or("typst: artifact not a js string")?;
            match k.as_str() {
                "metadata" => {
                    let artifact = self.builder.from_value(v)?;
                    metadata.metadata = artifact.meta;
                }
                "pages" => {
                    metadata.pages = self.parse_pages(&v)?;
                }
                _ => {
                    panic!("unknown key: {}", k);
                }
            }
        }

        Ok(metadata)
    }

    fn parse_pages(&self, val: &JsValue) -> Result<ItemArray<Frame>, JsValue> {
        let mut pages: ItemArray<Frame> = Default::default();
        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or("typst: not a js object")?)
            .iter()
            .map(convert_pair)
        {
            let k = k.as_string().ok_or("typst: artifact not a js string")?;
            match k.as_str() {
                "start" => {
                    pages.start = v.as_f64().ok_or_else(|| {
                        JsValue::from_str(&format!("typst: pages.start not a js number: {:?}", v))
                    })? as u32;
                }
                "size" => {
                    pages.size = v.as_f64().ok_or_else(|| {
                        JsValue::from_str(&format!("typst: pages.size not a js number: {:?}", v))
                    })? as u32;
                }
                _ => {
                    console_log!("unknown key pages: {}", k);
                    panic!("unknown key: {}", k);
                }
            }
        }
        Ok(pages)
    }
}

pub fn ir_artifact_header_from_js_string(val: String) -> Result<ArtifactHeader, JsValue> {
    let js_val = js_sys::JSON::parse(val.as_str()).unwrap();
    let metadata = IRArtifactHeaderJsBuilder::new().from_value(js_val);
    metadata
}
