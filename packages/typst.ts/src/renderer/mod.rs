use js_sys::Uint8Array;
use std::str::FromStr;
use tiny_skia as sk;
use typst::geom::{Color, RgbaColor};
use typst_ts_core::Artifact;
use wasm_bindgen::{prelude::*, Clamped};
use web_sys::ImageData;

use crate::pixmap;

use super::browser_world::TypstBrowserWorld;

pub(crate) mod builder;
pub use builder::TypstRendererBuilder;

pub(crate) mod render;

pub(crate) mod session;
pub use session::RenderSession;

#[wasm_bindgen]
pub struct RenderImageOptions {
    pixel_per_pt: Option<f32>,
    background_color: Option<String>,
}

#[wasm_bindgen]
impl RenderImageOptions {
    #[wasm_bindgen(constructor)]
    pub fn new() -> RenderImageOptions {
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
pub struct TypstRenderer {
    world: TypstBrowserWorld,
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn render(
        &mut self,
        artifact_content: String,
        options: Option<RenderImageOptions>,
    ) -> Result<ImageData, JsValue> {
        let mut ses = self.parse_artifact(artifact_content)?;

        ses.pixel_per_pt = options
            .as_ref()
            .and_then(|o| o.pixel_per_pt.clone())
            .unwrap_or(2.);

        ses.background_color = options
            .as_ref()
            .and_then(|o| o.background_color.clone())
            .unwrap_or("ffffff".to_string());

        let (prealloc, size) = self.render_to_image_internal(&ses)?;

        Ok(ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(prealloc.as_slice()),
            size.width,
            size.height,
        )?)
    }

    pub fn render_to_pdf(&mut self, artifact_content: String) -> Result<Uint8Array, JsValue> {
        Ok(Uint8Array::from(
            self.render_to_pdf_internal(artifact_content)?.as_slice(),
        ))
    }
}

impl TypstRenderer {
    pub fn new(world: TypstBrowserWorld) -> TypstRenderer {
        Self { world }
    }

    pub fn render_to_pdf_internal(&self, artifact_content: String) -> Result<Vec<u8>, String> {
        let document = self.parse_artifact(artifact_content)?;
        Ok(typst::export::pdf(&document.doc))
    }

    pub fn render_to_image_internal(
        &self,
        ses: &RenderSession,
    ) -> Result<(Vec<u8>, pixmap::IntSize), JsValue> {
        let (data_len, size) = {
            let size = ses.doc.pages[0].size();
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
        self.render_to_image_prealloc(&ses, &mut [prealloc.as_mut_slice()])?;

        Ok((prealloc, size))
    }

    pub fn render_to_image_prealloc(
        &self,
        ses: &RenderSession,
        buffers: &mut [&mut [u8]],
    ) -> Result<(), String> {
        let size = ses.doc.pages[0].size();
        let pxw = (ses.pixel_per_pt * (size.x.to_pt() as f32))
            .round()
            .max(1.0) as u32;
        let pxh = (ses.pixel_per_pt * (size.y.to_pt() as f32))
            .round()
            .max(1.0) as u32;
        let mut canvas = sk::PixmapMut::from_bytes(buffers[0], pxw, pxh).ok_or(format!(
            "failed to create canvas reference: {}x{}",
            pxw, pxh
        ))?;

        Ok(render::render(
            &mut canvas,
            &ses.doc.pages[0],
            ses.pixel_per_pt,
            Color::Rgba(RgbaColor::from_str(&ses.background_color)?),
        ))
    }

    pub fn parse_artifact(&self, artifact_content: String) -> Result<RenderSession, String> {
        // todo:
        // https://medium.com/@wl1508/avoiding-using-serde-and-deserde-in-rust-webassembly-c1e4640970ca
        let artifact: Artifact = serde_json::from_str(artifact_content.as_str()).unwrap();

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

        let document = artifact.to_document(&self.world.font_resolver);
        if document.pages.len() == 0 {
            return Err("no pages in artifact".into());
        }

        Ok(RenderSession::from_doc(document))
    }
}
