use js_sys::Uint8Array;
use std::str::FromStr;
use typst::geom::{Color, RgbaColor};
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;

use crate::RenderSessionManager;
use crate::{pixmap, renderer::session::RenderSessionOptions};

use crate::browser_world::TypstBrowserWorld;

pub(crate) mod artifact;
pub use artifact::ArtifactJsBuilder;

pub(crate) mod artifact_ir;
pub use artifact_ir::IRArtifactMetadataJsBuilder;

pub(crate) mod builder;
pub use builder::TypstRendererBuilder;

pub(crate) mod render;

pub(crate) mod session;
pub use session::RenderSession;

#[wasm_bindgen]
pub struct RenderPageImageOptions {
    page_off: usize,
}

#[wasm_bindgen]
impl RenderPageImageOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { page_off: 0 }
    }

    #[wasm_bindgen(getter)]
    pub fn page_off(&self) -> usize {
        self.page_off
    }

    #[wasm_bindgen(setter)]
    pub fn set_page_off(&mut self, page_off: usize) {
        self.page_off = page_off;
    }
}

#[wasm_bindgen]
pub struct TypstRenderer {
    session_mgr: RenderSessionManager,
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn render_page(
        &mut self,
        session: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> Result<ImageData, JsValue> {
        let (prealloc, size) = self.render_to_image_internal(session, options)?;

        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(prealloc.as_slice()),
            size.width,
            size.height,
        )?)
    }

    pub fn render_page_to_canvas(
        &mut self,
        ses: &RenderSession,
        canvas: &web_sys::CanvasRenderingContext2d,
        options: Option<RenderPageImageOptions>,
    ) -> Result<JsValue, JsValue> {
        let page_off = self.retrieve_page_off(ses, options)?;

        let mut worker = typst_ts_canvas_exporter::CanvasRenderTask::new(
            &canvas,
            &ses.doc,
            &ses.ligature_map,
            page_off,
            ses.pixel_per_pt,
            Color::Rgba(RgbaColor::from_str(&ses.background_color)?),
        );

        worker.render(&ses.doc.pages[page_off]);
        Ok(serde_wasm_bindgen::to_value(&worker.content).unwrap())
    }

    pub fn render_to_pdf(&mut self, artifact_content: &[u8]) -> Result<Uint8Array, JsValue> {
        let session = self.session_from_artifact(artifact_content)?;
        self.render_to_pdf_in_session(&session)
    }

    pub fn render_to_pdf_in_session(
        &mut self,
        session: &RenderSession,
    ) -> Result<Uint8Array, JsValue> {
        Ok(Uint8Array::from(
            self.render_to_pdf_internal(&session)?.as_slice(),
        ))
    }

    pub fn create_session(
        &self,
        artifact_content: &[u8],
        options: Option<RenderSessionOptions>,
    ) -> Result<RenderSession, JsValue> {
        self.session_mgr
            .create_session_internal(artifact_content, options)
    }

    pub fn load_page(
        &self,
        session: &mut RenderSession,
        page_number: usize,
        page_content: String,
    ) -> Result<(), JsValue> {
        self.session_mgr
            .load_page(session, page_number, page_content)
    }
}

#[cfg(not(feature = "render_raster"))]
impl TypstRenderer {
    pub fn render_to_image_internal(
        &self,
        _ses: &RenderSession,
        _options: Option<RenderPageImageOptions>,
    ) -> Result<(Vec<u8>, pixmap::IntSize), JsValue> {
        Err("render_raster feature is not enabled".into())
    }
}

#[cfg(feature = "render_raster")]
use tiny_skia as sk;

#[cfg(feature = "render_raster")]
impl TypstRenderer {
    pub fn render_to_image_internal(
        &self,
        ses: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> Result<(Vec<u8>, pixmap::IntSize), JsValue> {
        let page_off = self.retrieve_page_off(ses, options)?;

        let (data_len, size) = {
            let size = ses.doc.pages[page_off].size();
            let pxw = (ses.pixel_per_pt * (size.x.to_pt() as f32))
                .round()
                .max(1.0) as u32;
            let pxh = (ses.pixel_per_pt * (size.y.to_pt() as f32))
                .round()
                .max(1.0) as u32;
            let size = pixmap::IntSize {
                width: pxw,
                height: pxh,
            };
            let data_len =
                pixmap::data_len_for_size(size).ok_or("cannot compute data_len_for_size")?;
            (data_len, size)
        };

        let mut prealloc = vec![0; data_len];
        self.render_to_image_prealloc(&ses, page_off, &mut prealloc.as_mut_slice())?;

        Ok((prealloc, size))
    }

    pub fn render_to_image_prealloc(
        &self,
        ses: &RenderSession,
        page_off: usize,
        buffer: &mut [u8],
    ) -> Result<(), String> {
        let size = ses.doc.pages[page_off].size();
        let pxw = (ses.pixel_per_pt * (size.x.to_pt() as f32))
            .round()
            .max(1.0) as u32;
        let pxh = (ses.pixel_per_pt * (size.y.to_pt() as f32))
            .round()
            .max(1.0) as u32;
        let mut canvas = sk::PixmapMut::from_bytes(buffer, pxw, pxh).ok_or(format!(
            "failed to create canvas reference: {}x{}",
            pxw, pxh
        ))?;

        #[cfg(debug)]
        {
            use super::utils::console_log;
            use web_sys::console;
            let _ = console::log_0;
            console_log!(
                "{} pages to render. page_off: {:?}, page_hash {:?}",
                ses.doc.pages.len(),
                page_off,
                typst_ts_core::typst_affinite_hash(&ses.doc.pages[page_off]),
            );
        }

        // contribution: 850KB
        Ok(typst_ts_raster_exporter::render(
            &mut canvas,
            &ses.doc.pages[page_off],
            ses.pixel_per_pt,
            Color::Rgba(RgbaColor::from_str(&ses.background_color)?),
        ))
    }
}

impl TypstRenderer {
    pub fn new(world: TypstBrowserWorld) -> TypstRenderer {
        Self {
            session_mgr: RenderSessionManager::new(world.font_resolver),
        }
    }

    fn retrieve_page_off(
        &self,
        ses: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> Result<usize, JsValue> {
        if ses.doc.pages.is_empty() {
            // todo: better error
            return Err("no pages in session".into());
        }

        let page_off = options.as_ref().map(|o| o.page_off).unwrap_or(0);

        if page_off < ses.doc.pages.len() {
            if page_off == ses.pages_info.pages[page_off].page_off {
                return Ok(page_off);
            }
        }

        for (i, page_info) in ses.pages_info.pages.iter().enumerate() {
            if page_info.page_off == page_off {
                return Ok(i);
            }
        }

        return Err(format!("page_off {} not found in pages_info", page_off).into());
    }

    pub fn render_to_pdf_internal(&self, _session: &RenderSession) -> Result<Vec<u8>, String> {
        // contribution 510KB
        // Ok(typst::export::pdf(&session.doc))
        Err("pdf disabled".into())
    }

    pub fn session_from_artifact(&self, artifact_content: &[u8]) -> Result<RenderSession, JsValue> {
        self.session_mgr
            .session_from_artifact(artifact_content, "js")
    }
}
