use std::sync::{Arc, RwLock};

use js_sys::Uint8Array;
use typst::font::{FontFlags, FontInfo as TypstFontInfo, FontVariant};
use typst::geom::Abs;
use typst_ts_core::artifact::doc::Frame;
use typst_ts_core::artifact_ir::{Artifact as IRArtifact, ArtifactHeader as IRArtifactHeader};
use typst_ts_core::{font::FontResolverImpl, Artifact, ArtifactMeta, FontResolver};
use wasm_bindgen::prelude::*;

use super::artifact::{artifact_from_js_string, page_from_js_string};
use super::artifact_ir::ir_artifact_header_from_js_string;

#[wasm_bindgen]
#[derive(Default, Debug)]
pub struct RenderSessionOptions {
    pub(crate) pixel_per_pt: Option<f32>,
    pub(crate) background_color: Option<String>,
    pub(crate) format: Option<String>,
}

#[wasm_bindgen]
impl RenderSessionOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenderSessionOptions {
        Self {
            pixel_per_pt: None,
            background_color: None,
            format: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> Option<f32> {
        self.pixel_per_pt.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.pixel_per_pt = Some(pixel_per_pt);
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> Option<String> {
        self.background_color.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_background_color(&mut self, background_color: String) {
        self.background_color = Some(background_color);
    }

    #[wasm_bindgen(getter)]
    pub fn format(&self) -> Option<String> {
        self.format.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_format(&mut self, format: String) {
        self.format = Some(format);
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PageInfo {
    pub(crate) page_off: usize,
    pub(crate) width: Abs,
    pub(crate) height: Abs,
}

#[wasm_bindgen]
impl PageInfo {
    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(getter)]
    pub fn width_pt(&self) -> f64 {
        self.width.to_pt()
    }

    #[wasm_bindgen(getter)]
    pub fn height_pt(&self) -> f64 {
        self.height.to_pt()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PagesInfo {
    pub(crate) pages: Vec<PageInfo>,
}

#[wasm_bindgen]
impl PagesInfo {
    #[wasm_bindgen(getter)]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn page_by_number(&self, num: usize) -> Option<PageInfo> {
        for page in &self.pages {
            if page.page_off == num {
                return Some(page.clone());
            }
        }
        None
    }

    pub fn page(&self, i: usize) -> PageInfo {
        self.pages[i].clone()
    }
}

pub type LigatureMap = std::collections::HashMap<
    (String, FontVariant, FontFlags),
    std::collections::HashMap<u16, std::string::String>,
>;

#[wasm_bindgen]
pub struct RenderSession {
    pub(crate) pixel_per_pt: f32,
    pub(crate) background_color: String,
    pub(crate) doc: typst::doc::Document,
    pub(crate) artifact_meta: ArtifactMeta,
    pub(crate) pages_info: PagesInfo,
    pub(crate) ligature_map: LigatureMap,
}

#[wasm_bindgen]
impl RenderSession {
    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> f32 {
        self.pixel_per_pt.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> String {
        self.background_color.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn pages_info(&self) -> PagesInfo {
        self.pages_info.clone()
    }
}

impl RenderSession {
    pub(crate) fn from_artifact<T: FontResolver>(artifact: Artifact, font_resolver: &T) -> Self {
        let mut ligature_map = std::collections::HashMap::<
            (String, FontVariant, FontFlags),
            std::collections::HashMap<u16, std::string::String>,
        >::new();
        for font in &(artifact.meta).fonts {
            let font_info = TypstFontInfo {
                family: font.family.clone(),
                variant: font.variant,
                flags: FontFlags::from_bits(font.flags).unwrap(),
                coverage: font.coverage.clone(),
            };
            // todo: font alternative
            let idx = font_resolver
                .font_book()
                .select_fallback(Some(&font_info), font.variant, "0")
                .unwrap();
            let local_font = font_resolver.font(idx).unwrap();
            let font_info = local_font.info();

            ligature_map.insert(
                (font_info.family.clone(), font_info.variant, font_info.flags),
                std::collections::HashMap::from_iter(font.ligatures.iter().map(|s| s.clone())),
            );
        }
        let artifact_meta = artifact.meta.clone();
        let doc = artifact.to_document(font_resolver);

        let pages_info = PagesInfo {
            pages: {
                let mut pages = Vec::new();
                pages.reserve(doc.pages.len());
                for (i, page) in doc.pages.iter().enumerate() {
                    pages.push(PageInfo {
                        page_off: i,
                        width: page.size().x,
                        height: page.size().y,
                    });
                }
                pages
            },
        };

        Self {
            pixel_per_pt: 0.,
            background_color: "".to_string(),
            doc,
            artifact_meta,
            pages_info,
            ligature_map,
        }
    }

    pub(crate) fn from_ir_artifact<T: FontResolver>(
        artifact: IRArtifact,
        font_resolver: &T,
    ) -> Self {
        let mut ligature_map = std::collections::HashMap::<
            (String, FontVariant, FontFlags),
            std::collections::HashMap<u16, std::string::String>,
        >::new();
        for font in &(artifact).metadata.fonts {
            let font_info = TypstFontInfo {
                family: font.family.clone(),
                variant: font.variant,
                flags: FontFlags::from_bits(font.flags).unwrap(),
                coverage: font.coverage.clone(),
            };
            // todo: font alternative
            let idx = font_resolver
                .font_book()
                .select_fallback(Some(&font_info), font.variant, "0")
                .unwrap();
            let local_font = font_resolver.font(idx).unwrap();
            let font_info = local_font.info();

            ligature_map.insert(
                (font_info.family.clone(), font_info.variant, font_info.flags),
                std::collections::HashMap::from_iter(font.ligatures.iter().map(|s| s.clone())),
            );
        }
        let artifact_meta = artifact.metadata.clone();
        let doc = artifact.to_document(font_resolver);

        let pages_info = PagesInfo {
            pages: {
                let mut pages = Vec::new();
                pages.reserve(doc.pages.len());
                for (i, page) in doc.pages.iter().enumerate() {
                    pages.push(PageInfo {
                        page_off: i,
                        width: page.size().x,
                        height: page.size().y,
                    });
                }
                pages
            },
        };

        Self {
            pixel_per_pt: 0.,
            background_color: "".to_string(),
            doc,
            artifact_meta,
            pages_info,
            ligature_map,
        }
    }

    pub(crate) fn load_page<T: FontResolver>(
        &mut self,
        page_off: usize,
        frame: Frame,
        font_resolver: &T,
    ) {
        let mut artifact = Artifact {
            meta: self.artifact_meta.clone(),
            pages: Vec::new(),
        };
        artifact.pages.push(frame);
        let doc = artifact.to_document(font_resolver);
        let page = &doc.pages[0];
        let page_info = PageInfo {
            page_off,
            width: page.size().x,
            height: page.size().y,
        };

        let mut pages = self.pages_info.pages.clone();
        let idx = pages.iter().position(|p| p.page_off == page_off);
        if let Some(idx) = idx {
            pages[idx] = page_info;
            self.doc.pages[idx] = page.clone();
        } else {
            let idx = pages.iter().position(|p| p.page_off > page_off);
            if let Some(idx) = idx {
                pages.insert(idx, page_info);
                self.doc.pages.insert(idx, page.clone());
            } else {
                pages.push(page_info);
                self.doc.pages.push(page.clone());
            }
        }
        self.pages_info = PagesInfo { pages };
    }
}

#[wasm_bindgen]
pub struct RenderSessionManager {
    font_resolver: Arc<RwLock<FontResolverImpl>>,
}

#[wasm_bindgen]
impl RenderSessionManager {
    pub fn create_session(
        &self,
        artifact_content: Uint8Array,
        options: Option<RenderSessionOptions>,
    ) -> Result<RenderSession, JsValue> {
        self.create_session_internal(artifact_content.to_vec().as_slice(), options)
    }

    pub(crate) fn create_session_internal(
        &self,
        artifact_content: &[u8],
        options: Option<RenderSessionOptions>,
    ) -> Result<RenderSession, JsValue> {
        let format = options
            .as_ref()
            .and_then(|o| o.format.as_ref())
            .map(|f| f.as_str())
            .unwrap_or("js");
        let mut ses = self.session_from_artifact(artifact_content.to_vec().as_slice(), format)?;

        ses.pixel_per_pt = options
            .as_ref()
            .and_then(|o| o.pixel_per_pt.clone())
            .unwrap_or(2.);

        ses.background_color = options
            .as_ref()
            .and_then(|o| o.background_color.clone())
            .unwrap_or("ffffff".to_string());

        Ok(ses)
    }

    pub fn load_page(
        &self,
        session: &mut RenderSession,
        page_number: usize,
        page_content: String,
    ) -> Result<(), JsValue> {
        self.session_load_page(session, page_number, page_content, "js")?;
        Ok(())
    }
}

impl RenderSessionManager {
    pub fn new(fr: FontResolverImpl) -> Self {
        Self {
            font_resolver: Arc::new(RwLock::new(fr)),
        }
    }

    pub fn session_from_artifact(
        &self,
        artifact_content: &[u8],
        decoder: &str,
    ) -> Result<RenderSession, JsValue> {
        if decoder != "ir" {
            self.session_from_json_artifact(artifact_content, decoder)
        } else {
            self.session_from_ir_artifact(artifact_content)
        }
    }

    fn session_from_json_artifact(
        &self,
        artifact_content: &[u8],
        decoder: &str,
    ) -> Result<RenderSession, JsValue> {
        // 550KB -> 147KB
        // https://medium.com/@wl1508/avoiding-using-serde-and-deserde-in-rust-webassembly-c1e4640970ca
        let artifact: Artifact = match decoder {
            "js" => {
                let artifact: Artifact = artifact_from_js_string(
                    std::str::from_utf8(artifact_content).unwrap().to_string(),
                )?;

                artifact
            }

            #[cfg(feature = "serde_json")]
            "serde_json" => {
                let artifact: Artifact =
                    serde_json::from_str(std::str::from_utf8(artifact_content).unwrap()).unwrap();

                artifact
            }
            _ => {
                panic!("unknown decoder: {}", decoder);
            }
        };

        #[cfg(debug)]
        {
            use super::utils::console_log;
            console_log!(
                "{} pages to render. font info: {:?}",
                artifact.pages.len(),
                artifact
                    .fonts
                    .iter()
                    .map(|f| f.family.as_str()) // serde_json::to_string(f).unwrap())
                    .collect::<Vec<&str>>()
                    .join(", ")
            );
        }

        let font_resolver = self.font_resolver.read().unwrap();
        let session: RenderSession = RenderSession::from_artifact(artifact, &*font_resolver);
        Ok(session)
    }

    fn session_from_ir_artifact(&self, artifact_content: &[u8]) -> Result<RenderSession, JsValue> {
        use byteorder::{LittleEndian, ReadBytesExt};
        use std::io::Read;
        let mut reader = std::io::Cursor::new(artifact_content);
        let mut magic = [0; 4];
        reader.read(&mut magic).unwrap();
        assert_eq!(magic, ['I' as u8, 'R' as u8, 'A' as u8, 'R' as u8]);
        assert_eq!(reader.read_i32::<LittleEndian>().unwrap(), 1);
        let header_len = reader.read_u64::<LittleEndian>().unwrap();
        let mut header = vec![0; header_len as usize];
        reader.read_exact(&mut header).unwrap();
        let header = String::from_utf8(header).unwrap();

        let header: IRArtifactHeader = if cfg!(feature = "serde_json") {
            #[cfg(not(feature = "serde_json"))]
            panic!("serde_json feature is not enabled");
            #[cfg(feature = "serde_json")]
            {
                let result = serde_json::from_str(&header).unwrap();
                result
            }
        } else {
            ir_artifact_header_from_js_string(header).unwrap()
        };

        let mut buffer = vec![];
        reader.read_to_end(&mut buffer).unwrap();

        let artifact = IRArtifact {
            metadata: header.metadata,
            pages: header.pages,
            buffer,
        };

        let font_resolver = self.font_resolver.read().unwrap();
        let session = RenderSession::from_ir_artifact(artifact, &*font_resolver);
        Ok(session)
    }

    pub fn session_load_page(
        &self,
        session: &mut RenderSession,
        page_number: usize,
        page_content: String,
        decoder: &str,
    ) -> Result<(), JsValue> {
        // 550KB -> 147KB
        // https://medium.com/@wl1508/avoiding-using-serde-and-deserde-in-rust-webassembly-c1e4640970ca
        let frame: Frame = match decoder {
            "js" => {
                let frame: Frame = page_from_js_string(page_content)?;

                frame
            }

            #[cfg(feature = "serde_json")]
            "serde_json" => {
                let frame: Frame = serde_json::from_str(page_content.as_str()).unwrap();

                frame
            }
            _ => {
                panic!("unknown decoder: {}", decoder);
            }
        };

        let font_resolver = self.font_resolver.read().unwrap();
        session.load_page(page_number, frame, &*font_resolver);
        Ok(())
    }

    // todo: set return error to typst_ts_core::Error
    #[allow(unreachable_code)]
    pub fn session_from_artifact_internal(
        &self,
        _artifact_content: &[u8],
        decoder: &str,
    ) -> Result<RenderSession, String> {
        let _artifact: Artifact = match decoder {
            #[cfg(feature = "serde_json")]
            "serde_json" => {
                let artifact: Artifact = serde_json::from_slice(_artifact_content).unwrap();

                artifact
            }
            #[cfg(feature = "serde_rmp")]
            "serde_rmp" => {
                let artifact: Artifact = rmp_serde::from_slice(_artifact_content).unwrap();

                artifact
            }
            _ => {
                panic!("unknown decoder: {}", decoder);
            }
        };

        let font_resolver = self.font_resolver.read().unwrap();
        let session: RenderSession = RenderSession::from_artifact(_artifact, &*font_resolver);
        Ok(session)
    }
}
