use std::ops::Deref;

use tiny_skia as sk;

use reflexo::{
    error::prelude::*,
    hash::Fingerprint,
    vector::{
        incr::IncrDocClient,
        ir::{ImmutStr, LayoutRegionNode, Module, Page, Rect},
        vm::{RenderState, RenderVm},
    },
};

use crate::{set_transform, CanvasAction, CanvasPage, CanvasTask, DefaultExportFeature};

pub struct IncrementalCanvasExporter {
    pub pixel_per_pt: f32,
    pub fill: ImmutStr,
    pub pages: Vec<CanvasPage>,
}

impl Default for IncrementalCanvasExporter {
    fn default() -> Self {
        Self {
            pixel_per_pt: 3.,
            fill: "#ffffff".into(),
            pages: vec![],
        }
    }
}

impl IncrementalCanvasExporter {
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        // render the document
        let mut t = CanvasTask::<DefaultExportFeature>::default();

        let mut ct = t.fork_canvas_render_task(module);

        let pages = pages
            .iter()
            .enumerate()
            .map(|(idx, Page { content, size })| {
                if idx < self.pages.len() && self.pages[idx].content == *content {
                    return self.pages[idx].clone();
                }

                let state = RenderState::new_size(*size);
                CanvasPage {
                    content: *content,
                    elem: ct.render_item(state, content),
                    size: *size,
                }
            })
            .collect();
        self.pages = pages;
    }

    pub async fn flush_page(
        &mut self,
        idx: usize,
        canvas: &web_sys::CanvasRenderingContext2d,
        ts: sk::Transform,
    ) {
        let pg = &self.pages[idx];

        set_transform(canvas, ts);
        canvas.set_fill_style(&self.fill.as_ref().into());
        canvas.fill_rect(0., 0., pg.size.x.0 as f64, pg.size.y.0 as f64);

        pg.elem.realize(ts, canvas).await;
    }
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrCanvasDocClient {
    /// canvas state
    pub elements: IncrementalCanvasExporter,

    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    pub doc_view: Option<Vec<Page>>,
}

impl IncrCanvasDocClient {
    pub fn reset(&mut self) {}

    pub fn set_pixel_per_pt(&mut self, pixel_per_pt: f32) {
        self.elements.pixel_per_pt = pixel_per_pt;
    }

    pub fn set_fill(&mut self, fill: ImmutStr) {
        self.elements.fill = fill;
    }

    fn patch_delta(&mut self, kern: &IncrDocClient) {
        if let Some(layout) = &kern.layout {
            let pages = layout.pages(&kern.doc.module);
            if let Some(pages) = pages {
                self.elements
                    .interpret_changes(pages.module(), pages.pages());
            }
        }
    }

    /// Render the document in the given window.
    pub async fn render_in_window(
        &mut self,
        kern: &mut IncrDocClient,
        canvas: &web_sys::CanvasRenderingContext2d,
        rect: Rect,
    ) {
        const NULL_PAGE: Fingerprint = Fingerprint::from_u128(1);

        self.patch_delta(kern);

        // prepare an empty page for the pages that are not rendered

        // get previous doc_view
        // it is exact state of the current DOM.
        let prev_doc_view = self.doc_view.take().unwrap_or_default();

        // render next doc_view
        // for pages that is not in the view, we use empty_page
        // otherwise, we keep document layout
        let mut page_off: f32 = 0.;
        let mut next_doc_view = vec![];
        if let Some(t) = &kern.layout {
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            for page in pages.iter() {
                page_off += page.size.y.0;
                if page_off < rect.lo.y.0 || page_off - page.size.y.0 > rect.hi.y.0 {
                    next_doc_view.push(Page {
                        content: NULL_PAGE,
                        size: page.size,
                    });
                    continue;
                }

                next_doc_view.push(page.clone());
            }
        }

        let s = self.elements.pixel_per_pt;
        let ts = sk::Transform::from_scale(s, s);

        // accumulate offset_y
        let mut offset_y = 0.;
        for (idx, y) in next_doc_view.iter().enumerate() {
            let x = prev_doc_view.get(idx);
            if x.is_none() || (x.unwrap() != y && y.content != NULL_PAGE) {
                let ts = ts.pre_translate(0., offset_y);
                self.elements.flush_page(idx, canvas, ts).await;
            }
            offset_y += y.size.y.0;
        }
    }

    /// Render the document in the given window.
    pub async fn render_page_in_window(
        &mut self,
        kern: &mut IncrDocClient,
        canvas: &web_sys::CanvasRenderingContext2d,
        idx: usize,
        _rect: Rect,
    ) -> ZResult<()> {
        self.patch_delta(kern);

        if idx >= self.elements.pages.len() {
            Err(error_once!("Renderer.OutofPageRange", idx: idx))?;
        }

        let s = self.elements.pixel_per_pt;
        let ts = sk::Transform::from_scale(s, s);
        self.elements.flush_page(idx, canvas, ts).await;

        Ok(())
    }
}
