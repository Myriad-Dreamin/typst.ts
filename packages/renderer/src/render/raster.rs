use std::str::FromStr;
use typst::geom::{Color, RgbaColor};
use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

use crate::{RenderPageImageOptions, RenderSession, TypstRenderer};

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
