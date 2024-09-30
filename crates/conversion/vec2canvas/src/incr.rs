use tiny_skia as sk;

use reflexo::{
    error::prelude::*,
    vector::{
        incr::IncrDocClient,
        ir::{ImmutStr, Module, Page, Rect},
        vm::RenderVm,
    },
};

use crate::{set_transform, CanvasDevice, CanvasOp, CanvasPage, CanvasTask, DefaultExportFeature};

/// Incremental pass from vector to canvas
pub struct IncrVec2CanvasPass {
    /// Canvas's pixel per point
    pub pixel_per_pt: f32,
    /// Fills background color with a css color string
    /// Default is white.
    ///
    /// Note: If the string is empty, the background is transparent.
    pub fill: ImmutStr,
    /// Holds a sequence of canvas pages that are rendered
    pub pages: Vec<CanvasPage>,
}

impl Default for IncrVec2CanvasPass {
    fn default() -> Self {
        Self {
            pixel_per_pt: 3.,
            fill: "#ffffff".into(),
            pages: vec![],
        }
    }
}

impl IncrVec2CanvasPass {
    /// Interprets the changes in the given module and pages.
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        // render the document
        let mut t = CanvasTask::<DefaultExportFeature>::default();

        let mut ct = t.fork_canvas_render_task(module);

        let pages: Vec<CanvasPage> = pages
            .iter()
            .enumerate()
            .map(|(idx, Page { content, size })| {
                if idx < self.pages.len() && self.pages[idx].content == *content {
                    return self.pages[idx].clone();
                }

                CanvasPage {
                    content: *content,
                    elem: ct.render_item(content),
                    size: *size,
                }
            })
            .collect();

        // let ts = sk::Transform::from_scale(self.pixel_per_pt, self.pixel_per_pt);
        // for page in pages.iter() {
        //     page.elem.prepare(ts);
        // }

        // web_sys::console::log_1(&"interpret_changes".into());
        self.pages = pages;
    }

    /// Flushes a page to the canvas with the given transform.
    pub async fn flush_page(&mut self, idx: usize, canvas: &dyn CanvasDevice, ts: sk::Transform) {
        let pg = &self.pages[idx];

        if !set_transform(canvas, ts) {
            return;
        }
        canvas.set_fill_style(&self.fill.as_ref().into());
        canvas.fill_rect(0., 0., pg.size.x.0 as f64, pg.size.y.0 as f64);

        pg.elem.realize(ts, canvas).await;
    }
}

/// Maintains the state of the incremental rendering a canvas at client side
#[derive(Default)]
pub struct IncrCanvasDocClient {
    /// State of converting vector to canvas
    pub vec2canvas: IncrVec2CanvasPass,

    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    pub doc_view: Option<Vec<Page>>,
}

impl IncrCanvasDocClient {
    /// Reset the state of the incremental rendering.
    pub fn reset(&mut self) {}

    /// Set canvas's pixel per point
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.vec2canvas.pixel_per_pt = pixel_per_pt;
    }

    /// Set canvas's background color
    pub fn set_fill(&mut self, fill: ImmutStr) {
        self.vec2canvas.fill = fill;
    }

    fn patch_delta(&mut self, kern: &IncrDocClient) {
        if let Some(layout) = &kern.layout {
            let pages = layout.pages(&kern.doc.module);
            if let Some(pages) = pages {
                self.vec2canvas
                    .interpret_changes(pages.module(), pages.pages());
            }
        }
    }

    /// Render a specific page of the document in the given window.
    pub async fn render_page_in_window(
        &mut self,
        kern: &mut IncrDocClient,
        canvas: &dyn CanvasDevice,
        idx: usize,
        _rect: Rect,
    ) -> ZResult<()> {
        self.patch_delta(kern);

        if idx >= self.vec2canvas.pages.len() {
            Err(error_once!("Renderer.OutofPageRange", idx: idx))?;
        }

        let s = self.vec2canvas.pixel_per_pt;
        let ts = sk::Transform::from_scale(s, s);
        self.vec2canvas.flush_page(idx, canvas, ts).await;

        Ok(())
    }
}
