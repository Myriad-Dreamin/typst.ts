use std::{future::Future, pin::Pin};

use tiny_skia as sk;

use reflexo::{
    error::prelude::*,
    hash::Fingerprint,
    vector::{
        incr::IncrDocClient,
        ir::{ImmutStr, Module, Page, Point, Rect, Scalar},
        vm::RenderVm,
    },
};

use crate::{
    set_transform, CanvasDevice, CanvasOp, CanvasPage, CanvasRenderContext, CanvasTask,
    DefaultExportFeature,
};

/// Prepared canvas resources that can be awaited after the document locks are
/// released.
pub type CanvasResourcePrepareFuture = Pin<Box<dyn Future<Output = ()> + 'static>>;

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

        self.pages = pages;
    }

    /// Flushes a page to the canvas with the given transform.
    pub async fn flush_page(&mut self, idx: usize, canvas: &dyn CanvasDevice, ts: sk::Transform) {
        self.flush_page_in_window(idx, canvas, ts, None).await
    }

    /// Flushes a page to the canvas with an optional page-local render window.
    pub async fn flush_page_in_window(
        &mut self,
        idx: usize,
        canvas: &dyn CanvasDevice,
        ts: sk::Transform,
        rect: Option<Rect>,
    ) {
        let pg = &self.pages[idx];

        if !set_transform(canvas, ts) {
            return;
        }
        canvas.set_fill_style_str(self.fill.as_ref());
        let fill_rect = rect
            .and_then(|rect| intersect_rect(rect, page_rect(pg.size)))
            .unwrap_or_else(|| page_rect(pg.size));
        canvas.fill_rect(
            fill_rect.left().0 as f64,
            fill_rect.top().0 as f64,
            fill_rect.width().0 as f64,
            fill_rect.height().0 as f64,
        );

        let window = rect.and_then(|rect| transform_rect(rect, ts));
        pg.elem
            .realize(ts, canvas, CanvasRenderContext::new(window))
            .await;
    }

    /// Starts preparation of external resources used by the selected pages.
    pub fn prepare_pages(
        &mut self,
        indices: &[usize],
        ts: sk::Transform,
    ) -> Result<Option<CanvasResourcePrepareFuture>> {
        let mut futures: Vec<CanvasResourcePrepareFuture> = Vec::new();

        for &idx in indices {
            let Some(pg) = self.pages.get(idx) else {
                Err(error_once!("Renderer.OutofPageRange", idx: idx))?
            };

            if let Some(f) = pg.elem.prepare(ts) {
                futures.push(Box::pin(f));
            }
        }

        if futures.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Box::pin(async move {
                for future in futures {
                    future.await;
                }
            })))
        }
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

    /// Whether the vector document has received new data since the last canvas
    /// interpretation.
    doc_dirty: bool,

    /// Page fingerprints for the last internal resource prefetch task.
    prefetched_page_fingerprints: Vec<Fingerprint>,
    prefetched_pixel_per_pt: f32,
    resources_dirty: bool,
}

impl IncrCanvasDocClient {
    /// Reset the state of the incremental rendering.
    pub fn reset(&mut self) {
        self.doc_view = None;
        self.doc_dirty = true;
        self.prefetched_page_fingerprints.clear();
        self.prefetched_pixel_per_pt = 0.;
        self.resources_dirty = true;
    }

    /// Marks the vector document as changed by a remote delta.
    pub fn mark_delta_dirty(&mut self) {
        self.doc_dirty = true;
        self.resources_dirty = true;
    }

    /// Set canvas's pixel per point
    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        if (self.vec2canvas.pixel_per_pt - pixel_per_pt).abs() >= f32::EPSILON {
            self.resources_dirty = true;
        }
        self.vec2canvas.pixel_per_pt = pixel_per_pt;
    }

    /// Set canvas's background color
    pub fn set_fill(&mut self, fill: ImmutStr) {
        self.vec2canvas.fill = fill;
    }

    fn patch_delta(&mut self, kern: &IncrDocClient) {
        if !self.doc_dirty {
            return;
        }

        if let Some(layout) = &kern.layout {
            let pages = layout.pages(&kern.doc.module);
            if let Some(pages) = pages {
                let next_doc_view = pages.pages();
                if self.doc_view.as_deref() != Some(next_doc_view) {
                    self.vec2canvas
                        .interpret_changes(pages.module(), next_doc_view);
                    self.doc_view = Some(next_doc_view.to_vec());
                }

                self.doc_dirty = false;
            }
        }
    }

    fn prefetch_page_resources(&mut self) {
        let pixel_per_pt = self.vec2canvas.pixel_per_pt;
        let pixel_changed = (self.prefetched_pixel_per_pt - pixel_per_pt).abs() >= f32::EPSILON;
        if !self.resources_dirty && !pixel_changed {
            return;
        }

        let page_count = self.vec2canvas.pages.len();
        if page_count == 0 {
            self.prefetched_page_fingerprints.clear();
            self.prefetched_pixel_per_pt = pixel_per_pt;
            self.resources_dirty = false;
            return;
        }

        let mut page_fingerprints = Vec::with_capacity(page_count);
        let mut indices = Vec::new();
        for (idx, page) in self.vec2canvas.pages.iter().enumerate() {
            page_fingerprints.push(page.content);
            if pixel_changed || self.prefetched_page_fingerprints.get(idx) != Some(&page.content) {
                indices.push(idx);
            }
        }

        self.prefetched_page_fingerprints = page_fingerprints;
        self.prefetched_pixel_per_pt = pixel_per_pt;
        self.resources_dirty = false;

        if indices.is_empty() {
            return;
        }

        let ts = sk::Transform::from_scale(pixel_per_pt, pixel_per_pt);
        let Ok(Some(prepare)) = self.vec2canvas.prepare_pages(&indices, ts) else {
            return;
        };

        wasm_bindgen_futures::spawn_local(async move {
            prepare.await;
        });
    }

    /// Render a specific page of the document in the given window.
    pub async fn render_page_in_window(
        &mut self,
        kern: &mut IncrDocClient,
        canvas: &dyn CanvasDevice,
        idx: usize,
        rect: Rect,
    ) -> Result<()> {
        self.patch_delta(kern);
        self.prefetch_page_resources();

        if idx >= self.vec2canvas.pages.len() {
            Err(error_once!("Renderer.OutofPageRange", idx: idx))?;
        }

        let s = self.vec2canvas.pixel_per_pt;
        let ts = sk::Transform::from_scale(s, s);
        let rect = (!is_full_render_rect(rect)).then_some(rect);
        self.vec2canvas
            .flush_page_in_window(idx, canvas, ts, rect)
            .await;

        Ok(())
    }

    /// Prepare external resources for a set of pages before drawing them.
    pub fn prepare_page_resources(
        &mut self,
        kern: &mut IncrDocClient,
        indices: &[usize],
    ) -> Result<Option<CanvasResourcePrepareFuture>> {
        self.patch_delta(kern);

        let s = self.vec2canvas.pixel_per_pt;
        let ts = sk::Transform::from_scale(s, s);
        self.vec2canvas.prepare_pages(indices, ts)
    }
}

fn is_full_render_rect(rect: Rect) -> bool {
    rect.lo.x.0 <= -1.0 && rect.lo.y.0 <= -1.0 && rect.hi.x.0 >= 1e20 && rect.hi.y.0 >= 1e20
}

fn page_rect(size: Point) -> Rect {
    Rect {
        lo: Point::new(Scalar(0.), Scalar(0.)),
        hi: size,
    }
}

fn intersect_rect(a: Rect, b: Rect) -> Option<Rect> {
    let lo_x = a.left().0.max(b.left().0);
    let lo_y = a.top().0.max(b.top().0);
    let hi_x = a.right().0.min(b.right().0);
    let hi_y = a.bottom().0.min(b.bottom().0);

    if hi_x <= lo_x || hi_y <= lo_y {
        return None;
    }

    Some(Rect {
        lo: Point::new(Scalar(lo_x), Scalar(lo_y)),
        hi: Point::new(Scalar(hi_x), Scalar(hi_y)),
    })
}

fn transform_rect(rect: Rect, ts: sk::Transform) -> Option<Rect> {
    sk::Rect::from_ltrb(rect.left().0, rect.top().0, rect.right().0, rect.bottom().0)
        .and_then(|rect| rect.transform(ts))
        .map(From::from)
}
