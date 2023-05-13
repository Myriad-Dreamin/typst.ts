use super::artifact::{convert_pair, ArtifactJsBuilder};
use crate::utils::console_log;
use typst_ts_core::artifact_ir::{core::ItemArray, doc::Frame};
use typst_ts_core::artifact_ir::{Artifact, ArtifactHeader};
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

    pub fn parse_header(&self, val: JsValue) -> Result<ArtifactHeader, JsValue> {
        let mut metadata = ArtifactHeader::default();

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or("typst: not a js object")?)
            .iter()
            .map(convert_pair)
        {
            let k = k.as_string().ok_or("typst: artifact not a js string")?;
            match k.as_str() {
                "metadata" => {
                    let artifact = self.builder.parse_artifact(v)?;
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

impl Default for IRArtifactHeaderJsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn ir_artifact_header_from_js_string(val: String) -> Result<ArtifactHeader, JsValue> {
    let js_val = js_sys::JSON::parse(val.as_str()).unwrap();

    IRArtifactHeaderJsBuilder::new().parse_header(js_val)
}

pub fn ir_artifact_from_bin(artifact_content: &[u8]) -> Artifact {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Read;
    let mut reader = std::io::Cursor::new(artifact_content);
    let mut magic = [0; 4];
    reader.read_exact(&mut magic).unwrap();
    assert_eq!(magic, [b'I', b'R', b'A', b'R']);
    assert_eq!(reader.read_i32::<LittleEndian>().unwrap(), 1);
    let header_len = reader.read_u64::<LittleEndian>().unwrap();
    let mut header = vec![0; header_len as usize];
    reader.read_exact(&mut header).unwrap();
    let header = String::from_utf8(header).unwrap();

    let header: ArtifactHeader = if cfg!(feature = "serde_json") {
        #[cfg(not(feature = "serde_json"))]
        panic!("serde_json feature is not enabled");
        #[cfg(feature = "serde_json")]
        {
            serde_json::from_str(&header).unwrap()
        }
    } else {
        ir_artifact_header_from_js_string(header).unwrap()
    };

    let mut buffer = vec![];
    reader.read_to_end(&mut buffer).unwrap();

    Artifact {
        metadata: header.metadata,
        pages: header.pages,
        buffer,
    }
}
