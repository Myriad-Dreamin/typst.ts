use super::artifact::{convert_pair, ArtifactJsBuilder};
use crate::utils::console_log;
use typst_ts_core::artifact_ir::{
    core::ItemArray, doc::Frame, ArtifactMetadata as IRArtifactMetadata,
};
use wasm_bindgen::prelude::*;
use web_sys::console;

pub struct IRArtifactMetadataJsBuilder {
    builder: ArtifactJsBuilder,
}

impl IRArtifactMetadataJsBuilder {
    pub fn new() -> Self {
        Self {
            builder: ArtifactJsBuilder {},
        }
    }

    pub fn from_value(&mut self, val: JsValue) -> Result<IRArtifactMetadata, JsValue> {
        let mut metadata = IRArtifactMetadata {
            build: None,
            fonts: vec![],
            title: None,
            author: vec![],
            pages: Default::default(),
        };

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or("typst: not a js object")?)
            .iter()
            .map(convert_pair)
        {
            let k = k.as_string().ok_or("typst: artifact not a js string")?;
            match k.as_str() {
                "build" => {
                    metadata.build = Some(self.builder.parse_build_info(&v)?);
                }
                "pages" => {
                    metadata.pages = self.parse_pages(&v)?;
                }
                "fonts" => {
                    for elem in v.dyn_into::<js_sys::Array>()?.iter() {
                        metadata.fonts.push(self.builder.parse_font_info(elem)?);
                    }
                }
                "title" => {
                    metadata.title = if v.is_null() {
                        None
                    } else {
                        Some(v.as_string().ok_or_else(|| {
                            JsValue::from_str(&format!("typst: title not a js string: {:?}", v))
                        })?)
                    }
                }
                "author" => {
                    for arr in v
                        .dyn_ref::<js_sys::Array>()
                        .ok_or("typst: author not a array")?
                        .iter()
                    {
                        metadata.author.push(arr.as_string().ok_or_else(|| {
                            JsValue::from_str(&format!("typst: author not a js string: {:?}", v))
                        })?);
                    }
                }
                _ => {
                    console_log!("unknown key in meta: {}", k);
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

pub fn ir_artifact_metadata_from_js_string(val: String) -> Result<IRArtifactMetadata, JsValue> {
    let js_val = js_sys::JSON::parse(val.as_str()).unwrap();
    let metadata = IRArtifactMetadataJsBuilder::new().from_value(js_val);
    metadata
}
