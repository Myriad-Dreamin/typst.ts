use super::artifact::{convert_pair, ArtifactJsBuilder};
use typst_ts_core::artifact_ir::{core::ItemArray, doc::Frame};
use typst_ts_core::artifact_ir::{Artifact, ArtifactHeader};
use typst_ts_core::error::prelude::*;
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

    pub fn parse_header(&self, val: JsValue) -> ZResult<ArtifactHeader> {
        let mut metadata = ArtifactHeader::default();

        for (k, v) in self
            .builder
            .into_entries("ir_header", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.builder.to_string("artifact", &k)?;
            match k.as_str() {
                "metadata" => {
                    let artifact = self.builder.parse_artifact(v)?;
                    metadata.metadata = artifact.meta;
                }
                "pages" => {
                    metadata.pages = self.parse_pages(v)?;
                }
                _ => {
                    return Err(error_once!("artifact_ir.unknown_key", k: k));
                }
            }
        }

        Ok(metadata)
    }

    fn parse_pages(&self, val: JsValue) -> ZResult<ItemArray<Frame>> {
        let mut pages: ItemArray<Frame> = Default::default();
        for (k, v) in self
            .builder
            .into_entries("page", val)?
            .iter()
            .map(convert_pair)
        {
            let k = self.builder.to_string("pages", &k)?;
            match k.as_str() {
                "start" => {
                    pages.start = self.builder.to_f64("pages.start", &v)? as u32;
                }
                "size" => {
                    pages.size = self.builder.to_f64("pages.size", &v)? as u32;
                }
                _ => {
                    return Err(error_once!("artifact_ir.pages.unknown_key", k: k));
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

fn ir_artifact_header_from_js_string(val: String) -> ZResult<ArtifactHeader> {
    let js_val =
        js_sys::JSON::parse(val.as_str()).map_err(map_err("ArtifactIRBuilder.ParseJson"))?;

    IRArtifactHeaderJsBuilder::new()
        .parse_header(js_val)
        .map_err(wrap_err("ArtifactIRBuilder.HeaderFmt"))
}

pub fn ir_artifact_from_bin(artifact_content: &[u8]) -> ZResult<Artifact> {
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Read;
    let mut reader = std::io::Cursor::new(artifact_content);

    let mut magic = [0; 4];
    reader
        .read_exact(&mut magic)
        .map_err(map_err("ArtifactIRBuilder.BinReadMagic"))?;
    if magic != [b'I', b'R', b'A', b'R'] {
        return Err(error_once!(
            "ArtifactIRBuilder.InvaidMagic",
            expect: "IRAR",
            got: format!("{:?}", magic)
        ));
    }

    let file_cnt = reader
        .read_i32::<LittleEndian>()
        .map_err(map_err("ArtifactIRBuilder.FileCount"))?;
    if file_cnt != 1 {
        return Err(error_once!(
            "ArtifactIRBuilder.InvalidFileCount",
            expect: "1",
            got: file_cnt
        ));
    }

    let header_len = reader
        .read_u64::<LittleEndian>()
        .map_err(map_err("ArtifactIRBuilder.BinReadHeaderLength"))?;
    let mut header = vec![0; header_len as usize];
    reader
        .read_exact(&mut header)
        .map_err(map_err("ArtifactIRBuilder.BinReadHeader"))?;
    let header =
        String::from_utf8(header).map_err(map_string_err("ArtifactIRBuilder.HeaderEncoding"))?;

    let header: ArtifactHeader = if cfg!(feature = "serde_json") {
        #[cfg(not(feature = "serde_json"))]
        panic!("serde_json feature is not enabled");
        #[cfg(feature = "serde_json")]
        {
            serde_json::from_str(&header).map_err(map_string_err("ArtifactIRBuilder.HeaderFmt"))?
        }
    } else {
        ir_artifact_header_from_js_string(header)?
    };

    let rest_offset = artifact_content.len() - reader.position() as usize;

    Ok(Artifact::with_initializer(
        rest_offset,
        |buf_mut| {
            buf_mut.copy_from_slice(&artifact_content[reader.position() as usize..]);
        },
        header,
    ))
}
