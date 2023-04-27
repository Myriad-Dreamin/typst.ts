use std::sync::{Arc, RwLock};

use typst::font::{FontFlags, FontInfo as TypstFontInfo, FontVariant};
use typst::geom::Abs;
use typst_ts_core::{font::FontResolverImpl, Artifact, FontResolver};
use wasm_bindgen::prelude::*;

use crate::renderer::artifact::artifact_from_js_string;

#[wasm_bindgen]
pub struct RenderSessionOptions {
    pub(crate) pixel_per_pt: Option<f32>,
    pub(crate) background_color: Option<String>,
}

#[wasm_bindgen]
impl RenderSessionOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenderSessionOptions {
        Self {
            pixel_per_pt: None,
            background_color: None,
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
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PageInfo {
    pub(crate) width: Abs,
    pub(crate) height: Abs,
}

#[wasm_bindgen]
impl PageInfo {
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
    pub(crate) fn from_artifact<T: FontResolver>(
        artifact: typst_ts_core::Artifact,
        font_resolver: &T,
    ) -> Self {
        let mut ligature_map = std::collections::HashMap::<
            (String, FontVariant, FontFlags),
            std::collections::HashMap<u16, std::string::String>,
        >::new();
        for font in &(artifact).fonts {
            let font_info = TypstFontInfo {
                family: font.family.clone(),
                variant: font.variant,
                flags: font.flags,
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
        let doc = artifact.to_document(font_resolver);

        let pages_info = PagesInfo {
            pages: {
                let mut pages = Vec::new();
                pages.reserve(doc.pages.len());
                for page in doc.pages.iter() {
                    pages.push(PageInfo {
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
            pages_info,
            ligature_map,
        }
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
        artifact_content: String,
        options: Option<RenderSessionOptions>,
    ) -> Result<RenderSession, JsValue> {
        let mut ses = self.session_from_artifact(artifact_content, "js")?;

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
}

impl RenderSessionManager {
    pub fn new(fr: FontResolverImpl) -> Self {
        Self {
            font_resolver: Arc::new(RwLock::new(fr)),
        }
    }

    pub fn session_from_artifact(
        &self,
        artifact_content: String,
        decoder: &str,
    ) -> Result<RenderSession, JsValue> {
        // 550KB -> 147KB
        // https://medium.com/@wl1508/avoiding-using-serde-and-deserde-in-rust-webassembly-c1e4640970ca
        let artifact: Artifact = match decoder {
            "js" => {
                let artifact: Artifact = artifact_from_js_string(artifact_content)?;

                artifact
            }

            #[cfg(feature = "serde_json")]
            "serde_json" => {
                let artifact: Artifact = serde_json::from_str(artifact_content.as_str()).unwrap();

                artifact
            }
            _ => {
                panic!("unknown decoder: {}", decoder);
            }
        };

        #[cfg(debug)]
        {
            use super::utils::console_log;
            use web_sys::console;
            let _ = console::log_0;
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
        if session.doc.pages.len() == 0 {
            return Err("no pages in artifact".into());
        }
        Ok(session)
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
        if session.doc.pages.len() == 0 {
            return Err("no pages in artifact".into());
        }
        Ok(session)
    }
}
