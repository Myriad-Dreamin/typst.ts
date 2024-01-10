use std::ops::Deref;

use typst_ts_core::{
    error::prelude::*,
    hash::Fingerprint,
    vector::{
        incr::{IncrDocClient, IncrDocServer},
        ir::{FontItem, FontRef, LayoutRegionNode, Module, Page, VecItem},
    },
};
use web_sys::{wasm_bindgen::JsCast, Element, HtmlElement};

use crate::{
    canvas_backend::CanvasBackend, dom::DomPage, factory::XmlFactory, svg_backend::SvgBackend,
};

pub type IncrDOMDocServer = IncrDocServer;

// const NULL_PAGE: Fingerprint = Fingerprint::from_u128(1);

#[derive(Default)]
enum TrackMode {
    #[default]
    Document,
}

#[derive(Default, PartialEq, Eq)]
pub enum CheckoutMode {
    #[default]
    Full,
    Responsive,
}

pub enum DOMChanges {
    /// Change the element to track.
    Unmount(HtmlElement),
    /// Change the element to track.
    Mount(HtmlElement),
    /// Change viewport.
    Viewport(Option<tiny_skia::Rect>),
    /// Recalculate in/out responsive loop
    Recalc(bool),
}

#[derive(Debug, Clone)]
pub struct HookedElement {
    pub hooked: HtmlElement,
    pub resource_header: Element,
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrDomDocClient {
    tmpl: XmlFactory,
    stub: once_cell::sync::OnceCell<Element>,
    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    doc_view: Vec<DomPage>,
    /// Track mode.
    track_mode: TrackMode,
    /// The element to track.
    elem: Option<HookedElement>,
    /// The viewport.
    viewport: Option<tiny_skia::Rect>,

    /// Backend for rendering vector IR as SVG.
    svg_backend: SvgBackend,
    /// Backend for rendering vector IR as Canvas.
    canvas_backend: CanvasBackend,
}

impl IncrDomDocClient {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn stub(&self) -> &Element {
        self.stub.get_or_init(|| {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("typst-stub")
                .unwrap()
        })
    }

    fn checkout_pages<'b>(
        &mut self,
        kern: &'b mut IncrDocClient,
    ) -> impl ExactSizeIterator<Item = &'b Page> + 'b {
        // Check out the current document layout.
        let Some(t) = &kern.layout else {
            return [].iter();
        };

        let pages = match t {
            LayoutRegionNode::Pages(a) => &a.deref().1,
            _ => todo!(),
        };

        pages.as_slice().iter()
    }

    // todo: move to js world
    fn checkout_layout(&mut self, kern: &mut IncrDocClient, viewport: Option<tiny_skia::Rect>) {
        let layouts = kern.doc.layouts[0].by_scalar().unwrap();
        let mut layout = layouts.first().unwrap();

        if let Some(viewport) = viewport {
            // base scale = 2
            let base_cw = viewport.width();

            const EPS: f32 = 1e-2;

            if layout.0 .0 >= base_cw + EPS {
                let layout_alt = layouts.last().unwrap();

                if layout_alt.0 .0 + EPS > base_cw {
                    layout = layout_alt;
                } else {
                    for layout_alt in layouts {
                        if layout_alt.0 .0 < base_cw + EPS {
                            layout = layout_alt;
                            break;
                        }
                    }
                }
            }
        }

        let layout = layout.clone();
        kern.set_layout(layout.1.clone());
    }

    pub fn reset(&mut self) {}

    /// Render the document in the given window.
    pub async fn mount(&mut self, kern: &mut IncrDocClient, elem: HtmlElement) -> ZResult<()> {
        self.batch_dom_events(
            kern,
            vec![DOMChanges::Mount(elem), DOMChanges::Recalc(false)],
        )
        .await
    }

    /// Render the document in the given window.
    pub async fn rerender(
        &mut self,
        kern: &mut IncrDocClient,
        viewport: Option<tiny_skia::Rect>,
        is_responsive: bool,
    ) -> ZResult<()> {
        self.batch_dom_events(
            kern,
            vec![
                DOMChanges::Viewport(viewport),
                DOMChanges::Recalc(is_responsive),
            ],
        )
        .await
    }
}

impl IncrDomDocClient {
    /// Emit a batch of changes.
    pub async fn batch_dom_events(
        &mut self,
        kern: &mut IncrDocClient,
        changes: impl IntoIterator<Item = DOMChanges>,
    ) -> ZResult<()> {
        for change in changes {
            match change {
                DOMChanges::Unmount(elem) => {
                    if !matches!(self.elem, Some(ref e) if e.hooked == elem) {
                        return Err(error_once!("not mounted or mismatched"));
                    }

                    self.elem = None;
                }
                DOMChanges::Mount(elem) => {
                    if let Some(old_elem) = self.elem.as_ref() {
                        return Err(error_once!(
                            "already mounted to",
                            old_elem: format!("{:?}", old_elem.hooked)
                        ));
                    }

                    // create typst-svg-resources by string
                    elem.set_inner_html(
                        r#"<svg class="typst-svg-resources" viewBox="0 0 0 0" width="0" height="0" style="opacity: 0; position: absolute;"></svg>"#,
                    );
                    self.elem = Some(HookedElement {
                        hooked: elem.clone(),
                        resource_header: elem
                            .get_elements_by_tag_name("svg")
                            .item(0)
                            .unwrap()
                            .dyn_into()
                            .unwrap(),
                    });
                    self.svg_backend.reset();
                    self.canvas_backend.reset();
                }
                DOMChanges::Viewport(viewport) => {
                    self.viewport = viewport;

                    self.checkout_layout(kern, viewport);
                }
                DOMChanges::Recalc(is_responsive) => {
                    if let Some(elem) = &self.elem {
                        let checkout_mode = if is_responsive {
                            CheckoutMode::Responsive
                        } else {
                            CheckoutMode::Full
                        };

                        self.recalculate(kern, elem.clone(), checkout_mode).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn recalculate(
        &mut self,
        kern: &mut IncrDocClient,
        elem: HookedElement,
        checkout_mode: CheckoutMode,
    ) -> ZResult<()> {
        match self.track_mode {
            TrackMode::Document => {
                self.retrack_pages(kern, elem).await?;
            } // TrackMode::Pages => todo!(),
        }

        let mut ctx = DomContext {
            tmpl: self.tmpl.clone(),
            stub: self.stub().clone(),
            module: kern.module(),
            svg_backend: &mut self.svg_backend,
            canvas_backend: &mut self.canvas_backend,
            checkout_mode,
        };

        for page in self.doc_view.iter_mut() {
            page.recalculate(&mut ctx, self.viewport).await?;
        }

        Ok(())
    }

    async fn retrack_pages(
        &mut self,
        kern: &mut IncrDocClient,
        elem: HookedElement,
    ) -> ZResult<()> {
        // Checks out the current document layout.
        let pages = self.checkout_pages(kern);

        // Tracks the pages.
        if self.doc_view.len() > pages.len() {
            self.doc_view.truncate(pages.len());
        }
        for i in self.doc_view.len()..pages.len() {
            self.doc_view
                .push(DomPage::new_at(elem.hooked.clone(), self.tmpl.clone(), i));
        }
        for (page, data) in self.doc_view.iter_mut().zip(pages) {
            page.track_data(data);
        }

        // Populates the glyphs to dom so that they get rendered
        self.svg_backend.populate_glyphs(kern, &elem);

        Ok(())
    }
}

pub struct DomContext<'m, 'a> {
    tmpl: XmlFactory,
    stub: Element,
    pub svg_backend: &'a mut SvgBackend,
    pub canvas_backend: &'a mut CanvasBackend,
    pub module: &'m Module,
    pub checkout_mode: CheckoutMode,
}

impl<'m, 'a> DomContext<'m, 'a> {
    pub fn create_stub(&self) -> Element {
        self.stub.clone_node().unwrap().dyn_into().unwrap()
    }

    pub fn create_element(&self, html: &str) -> Element {
        self.tmpl.create_element(html)
    }

    pub fn get_item(&self, id: &Fingerprint) -> Option<&'m VecItem> {
        self.module.get_item(id)
    }

    pub fn get_font(&self, id: &FontRef) -> Option<&'m FontItem> {
        self.module.get_font(id)
    }
}
