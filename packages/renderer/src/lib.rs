// todo
#![allow(clippy::await_holding_lock)]

#[macro_use]
pub(crate) mod utils;

use js_sys::Uint8Array;
use std::collections::HashMap;
use std::str::FromStr;
use typst::geom::{Color, RgbaColor};
use typst_ts_canvas_exporter::{CanvasRenderTask, DefaultRenderFeature, RenderFeature};
use typst_ts_core::error::prelude::*;
use typst_ts_core::font::{
    FontGlyphProvider, FontResolverImpl, GlyphProvider, PartialFontGlyphProvider,
};
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

pub(crate) mod artifact;
pub use artifact::ArtifactJsBuilder;

pub(crate) mod artifact_ir;
pub use artifact_ir::IRArtifactHeaderJsBuilder;

pub(crate) mod builder;
pub use builder::TypstRendererBuilder;

pub(crate) mod render;

pub(crate) mod session;
pub use session::RenderSession;

pub use session::{RenderSessionManager, RenderSessionOptions};

#[wasm_bindgen]
#[derive(Debug, Default)]
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
    pub(crate) session_mgr: RenderSessionManager,
}

#[wasm_bindgen]
impl TypstRenderer {
    pub async fn render_page_to_canvas(
        &mut self,
        ses: &RenderSession,
        canvas: &web_sys::CanvasRenderingContext2d,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<JsValue> {
        Ok(self
            .render_page_to_canvas_internal::<DefaultRenderFeature>(ses, canvas, options)
            .await?
            .0)
    }

    pub fn render_to_pdf(&mut self, artifact_content: &[u8]) -> ZResult<Uint8Array> {
        let session = self.session_from_artifact(artifact_content)?;
        self.render_to_pdf_in_session(&session)
    }

    pub fn render_to_pdf_in_session(&mut self, session: &RenderSession) -> ZResult<Uint8Array> {
        Ok(Uint8Array::from(
            self.render_to_pdf_internal(session)?.as_slice(),
        ))
    }

    pub fn create_session(
        &self,
        artifact_content: &[u8],
        options: Option<RenderSessionOptions>,
    ) -> ZResult<RenderSession> {
        self.session_mgr
            .create_session_internal(artifact_content, options)
    }

    pub fn load_page(
        &self,
        session: &mut RenderSession,
        page_number: usize,
        page_content: String,
    ) -> ZResult<()> {
        self.session_mgr
            .load_page(session, page_number, page_content)
    }

    pub fn load_glyph_pack(&self, v: JsValue) -> ZResult<()> {
        let mut font_resolver = self.session_mgr.font_resolver.write().unwrap();
        font_resolver.add_glyph_packs(
            serde_wasm_bindgen::from_value(v).map_err(map_string_err("GlyphBundleFmt"))?,
        );
        Ok(())
    }
}

#[cfg(not(feature = "render_raster"))]
#[wasm_bindgen]
impl TypstRenderer {
    pub fn render_page(
        &mut self,
        _session: &RenderSession,
        _options: Option<RenderPageImageOptions>,
    ) -> ZResult<ImageData> {
        Err(error_once!("Renderer.RasterFeatureNotEnabled"))
    }
}

#[cfg(feature = "render_raster")]
#[wasm_bindgen]
impl TypstRenderer {
    pub fn render_page(
        &mut self,
        session: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<ImageData> {
        let buf = self.render_to_image_internal(session, options)?;
        let size = buf.size();

        ImageData::new_with_u8_clamped_array_and_sh(
            wasm_bindgen::Clamped(buf.as_slice()),
            size.width,
            size.height,
        )
        .map_err(error_once_map!("Renderer.CreateImageData"))
    }
}

#[cfg(feature = "render_raster")]
impl TypstRenderer {
    pub fn render_to_image_internal(
        &self,
        ses: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<typst_ts_raster_exporter::pixmap::PixmapBuffer> {
        let page_off = self.retrieve_page_off(ses, options)?;

        let canvas_size = ses.doc.pages[page_off].size();
        let mut canvas =
            typst_ts_raster_exporter::pixmap::PixmapBuffer::for_size(canvas_size, ses.pixel_per_pt)
                .ok_or_else(|| {
                    error_once!("Renderer.CannotCreatePixmap",
                        width: canvas_size.x.to_pt(), height: canvas_size.y.to_pt(),
                    )
                })?;

        self.render_to_image_prealloc(ses, page_off, &mut canvas.as_canvas_mut())?;

        Ok(canvas)
    }

    pub fn render_to_image_prealloc(
        &self,
        ses: &RenderSession,
        page_off: usize,
        canvas: &mut tiny_skia::PixmapMut,
    ) -> ZResult<()> {
        // contribution: 850KB
        typst_ts_raster_exporter::render(
            canvas,
            &ses.doc.pages[page_off],
            ses.pixel_per_pt,
            Color::Rgba(
                RgbaColor::from_str(&ses.background_color)
                    .map_err(map_err("Renderer.InvalidBackgroundColor"))?,
            ),
        );
        Ok(())
    }
}

impl TypstRenderer {
    pub fn new(font_resolver: FontResolverImpl) -> TypstRenderer {
        Self {
            session_mgr: RenderSessionManager::new(font_resolver),
        }
    }

    fn retrieve_page_off(
        &self,
        ses: &RenderSession,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<usize> {
        if ses.doc.pages.is_empty() {
            return Err(error_once!("Renderer.SessionDocNotPages"));
        }

        let page_off = options.as_ref().map(|o| o.page_off).unwrap_or(0);
        if page_off < ses.doc.pages.len() && page_off == ses.pages_info.pages[page_off].page_off {
            return Ok(page_off);
        }

        for (i, page_info) in ses.pages_info.pages.iter().enumerate() {
            if page_info.page_off == page_off {
                return Ok(i);
            }
        }

        Err(error_once!(
            "Renderer.SessionPageNotFound",
            offset: page_off
        ))
    }

    pub fn render_to_pdf_internal(&self, _session: &RenderSession) -> ZResult<Vec<u8>> {
        // contribution 510KB
        // Ok(typst::export::pdf(&session.doc))
        Err(error_once!("Renderer.PdfFeatureNotEnabled"))
    }

    pub async fn render_page_to_canvas_internal<Feat: RenderFeature>(
        &mut self,
        ses: &RenderSession,
        canvas: &web_sys::CanvasRenderingContext2d,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<(JsValue, Option<HashMap<String, f64>>)> {
        let page_off = self.retrieve_page_off(ses, options)?;

        let perf_events = if Feat::ENABLE_TRACING {
            Some(elsa::FrozenMap::<&'static str, Box<f64>>::default())
        } else {
            None
        };

        let mut worker = CanvasRenderTask::<Feat>::new(
            canvas,
            &ses.doc,
            page_off,
            ses.pixel_per_pt,
            Color::Rgba(
                RgbaColor::from_str(&ses.background_color)
                    .map_err(map_err("Renderer.InvalidBackgroundColor"))?,
            ),
        )?;

        let def_provider = GlyphProvider::new(FontGlyphProvider::default());
        let partial_providier =
            PartialFontGlyphProvider::new(def_provider, self.session_mgr.font_resolver.clone());

        worker.set_glyph_provider(GlyphProvider::new(partial_providier));

        crate::utils::console_log!("use partial font glyph provider");

        if let Some(perf_events) = perf_events.as_ref() {
            worker.set_perf_events(perf_events)
        };

        worker.render(&ses.doc.pages[page_off]).await?;

        Ok((
            serde_wasm_bindgen::to_value(&worker.content)
                .map_err(map_into_err::<JsValue, _>("Renderer.EncodeContent"))?,
            perf_events.map(|perf_events| {
                perf_events
                    .into_map()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect()
            }),
        ))
    }

    pub fn session_from_artifact(&self, artifact_content: &[u8]) -> ZResult<RenderSession> {
        self.session_mgr
            .session_from_artifact(artifact_content, "js")
    }
}

#[cfg(test)]
mod tests {
    use typst::util::Buffer;

    use super::{TypstRenderer, TypstRendererBuilder};
    use std::path::PathBuf;

    fn artifact_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fuzzers/corpora/math/main.artifact.json")
    }

    pub fn get_renderer() -> TypstRenderer {
        let mut root_path = PathBuf::new();
        root_path.push(".");

        let mut builder = TypstRendererBuilder::new().unwrap();

        // todo: prepare font files for test
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_R.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RB.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RBI.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/LinLibertine_RI.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Book.otf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NewCMMath-Regular.otf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Bold.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-BoldItalic.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Italic.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/InriaSerif-Regular.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/Roboto-Regular.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NotoSerifCJKsc-Regular.otf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-Oblique.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-BoldOblique.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/DejaVuSansMono-Bold.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/TwitterColorEmoji.ttf"
        )));
        builder.add_raw_font_internal(Buffer::from_static(include_bytes!(
            "../../../assets/fonts/NotoColorEmoji.ttf"
        )));

        pollster::block_on(builder.build()).unwrap()
    }

    #[test]
    fn test_render_document() {
        let renderer = get_renderer();

        let artifact_content = std::fs::read(artifact_path()).unwrap();

        let mut ses = renderer
            .session_mgr
            .session_from_artifact(artifact_content.as_slice(), "serde_json")
            .unwrap();
        ses.pixel_per_pt = 2.;
        ses.background_color = "ffffff".to_string();

        renderer.render_to_image_internal(&ses, None).unwrap();
    }
}
