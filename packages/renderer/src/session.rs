#[cfg(feature = "render_canvas")]
use std::sync::{Arc, Mutex};

use js_sys::Uint8Array;
#[cfg(feature = "render_canvas")]
use typst_ts_canvas_exporter::IncrCanvasDocClient;
use typst_ts_core::error::prelude::*;
use typst_ts_svg_exporter::ir::Scalar;
use wasm_bindgen::prelude::*;

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
        self.pixel_per_pt
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
    pub(crate) width: f64,
    pub(crate) height: f64,
}

#[wasm_bindgen]
impl PageInfo {
    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(getter)]
    pub fn width_pt(&self) -> f64 {
        self.width
    }

    #[wasm_bindgen(getter)]
    pub fn height_pt(&self) -> f64 {
        self.height
    }
}

#[wasm_bindgen]
#[derive(Clone, Default)]
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

    pub fn width(&self) -> f32 {
        self.pages
            .iter()
            .map(|s| Scalar(s.width as f32))
            .max()
            .unwrap_or_default()
            .0
    }

    pub fn height(&self) -> f32 {
        self.pages.iter().map(|s| s.height as f32).sum()
    }
}

#[derive(Default)]
#[wasm_bindgen]
pub struct RenderSession {
    pub(crate) pixel_per_pt: f32,
    pub(crate) background_color: String,
    #[cfg(feature = "render_canvas")]
    pub(crate) client: Arc<Mutex<IncrCanvasDocClient>>,
    pub(crate) pages_info: PagesInfo,
}

#[wasm_bindgen]
impl RenderSession {
    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> f32 {
        self.pixel_per_pt
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

#[cfg(feature = "render_canvas")]
impl RenderSession {
    pub fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
        use typst_ts_core::vector::stream::BytesModuleStream;

        let delta = BytesModuleStream::from_slice(delta).checkout_owned();

        #[cfg(feature = "debug_delta_update")]
        crate::utils::console_log!(
            "module counts: {:?},{:?},{:?}",
            delta.glyphs.len(),
            delta.item_pack.0.len(),
            delta.layouts.len()
        );

        let mut client = self.client.lock().unwrap();
        client.merge_delta(delta);

        let pages_info = PagesInfo {
            pages: {
                let mut pages = Vec::with_capacity(client.elements.pages.len());
                for (i, (_, size)) in client.elements.pages.iter().enumerate() {
                    pages.push(PageInfo {
                        page_off: i,
                        width: size.x.0 as f64,
                        height: size.y.0 as f64,
                    });
                }
                pages
            },
        };

        self.pages_info = pages_info;
        Ok(())
    }
}

#[wasm_bindgen]
pub struct RenderSessionManager {}

impl Default for RenderSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl RenderSessionManager {
    pub fn create_session(
        &self,
        artifact_content: Uint8Array,
        options: Option<RenderSessionOptions>,
    ) -> ZResult<RenderSession> {
        self.create_session_internal(artifact_content.to_vec().as_slice(), options)
    }

    pub(crate) fn create_session_internal(
        &self,
        artifact_content: &[u8],
        options: Option<RenderSessionOptions>,
    ) -> ZResult<RenderSession> {
        let format = options
            .as_ref()
            .and_then(|o| o.format.as_ref())
            .map(|f| f.as_str())
            .unwrap_or("vector");
        let mut ses = self.session_from_artifact(artifact_content.to_vec().as_slice(), format)?;

        ses.pixel_per_pt = options.as_ref().and_then(|o| o.pixel_per_pt).unwrap_or(2.);

        ses.background_color = options
            .as_ref()
            .and_then(|o| o.background_color.clone())
            .unwrap_or("ffffff".to_string());

        Ok(ses)
    }
}

impl RenderSessionManager {
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn session_from_artifact(
        &self,
        _artifact_content: &[u8],
        decoder: &str,
    ) -> ZResult<RenderSession> {
        // todo: share session between renderers
        #[cfg(feature = "render_canvas")]
        if decoder == "vector" {
            return self.session_from_vector_artifact(_artifact_content);
        }

        if decoder == "serde_json" || decoder == "js" || decoder == "ir" {
            Err(error_once!("deprecated format are removal in v0.4.0"))?
        }

        Err(error_once!("Renderer.UnsupportedDecoder", decoder: decoder))
    }

    #[cfg(feature = "render_canvas")]
    fn session_from_vector_artifact(&self, artifact_content: &[u8]) -> ZResult<RenderSession> {
        let mut session = RenderSession::default();
        session.merge_delta(artifact_content)?;
        Ok(session)
    }
}
