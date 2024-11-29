use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use js_sys::Reflect;
use reflexo::error::prelude::*;
use reflexo::hash::Fingerprint;
use reflexo::vector::ir::{FontItem, FontRef, LayoutRegionNode, Module, Page, VecItem};
use reflexo_typst2vec::incr::{IncrDocClient, IncrDocServer};
use reflexo_vec2canvas::CanvasOp;
use wasm_bindgen::prelude::*;
use web_sys::{wasm_bindgen::JsCast, Element, HtmlElement};

use crate::{
    canvas_backend::CanvasBackend, dom::DomPage, factory::XmlFactory,
    semantics_backend::SemanticsBackend, svg_backend::SvgBackend,
};

pub type IncrDOMDocServer = IncrDocServer;

#[derive(Default)]
enum TrackMode {
    #[default]
    Document,
}

#[derive(Debug, Clone)]
pub struct HookedElement {
    pub hooked: HtmlElement,
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
#[wasm_bindgen]
pub struct IncrDomDocClient {
    /// underlying communication client model
    client: Option<Arc<Mutex<IncrDocClient>>>,
    tmpl: XmlFactory,
    stub: std::sync::OnceLock<Element>,
    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    doc_view: Vec<DomPage>,
    /// The element to track.
    elem: Option<HookedElement>,
    /// The viewport.
    viewport: Option<tiny_skia::Rect>,
    /// populate glyphs callback
    populate_glyphs: Option<js_sys::Function>,

    /// Backend for rendering vector IR as SVG.
    svg_backend: SvgBackend,
    /// Backend for rendering vector IR as Canvas.
    canvas_backend: CanvasBackend,
    /// Backend for rendering vector IR as HTML Semantics.
    semantics_backend: SemanticsBackend,
}

const STAGE_LAYOUT: u8 = 0;
const STAGE_SVG: u8 = 1;
const STAGE_SEMANTICS: u8 = 2;
const STAGE_PREPARE_CANVAS: u8 = 3;
const STAGE_CANVAS: u8 = 4;

#[wasm_bindgen]
impl IncrDomDocClient {
    pub fn bind_functions(&mut self, functions: JsValue) {
        // let populate_glyphs = functions.get("populateGlyphs").unwrap();
        let populate_glyphs = Reflect::get(&functions, &"populateGlyphs".into()).unwrap();
        self.populate_glyphs = Some(populate_glyphs.dyn_into().unwrap());
    }

    /// Relayout the document in the given window.
    pub async fn relayout(&mut self, x: f32, y: f32, w: f32, h: f32) -> ZResult<bool> {
        // todo: overflow
        let viewport = Some(tiny_skia::Rect::from_xywh(x, y, w, h).unwrap());

        let kern = self.client.clone().unwrap();
        let mut kern = kern.lock().unwrap();

        self.checkout_layout(&mut kern, viewport);
        let viewport_dirty = self.viewport != viewport;
        self.viewport = viewport;

        let page_dirty = self.retrack_pages(&mut kern, self.elem.clone().unwrap())?;

        Ok(viewport_dirty || page_dirty)
    }

    pub fn need_repaint(
        &mut self,
        page_num: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        stage: u8,
    ) -> ZResult<bool> {
        // todo: overflow
        let viewport = Some(tiny_skia::Rect::from_xywh(x, y, w, h).unwrap());
        #[cfg(feature = "debug_recalc_stage")]
        web_sys::console::log_1(
            &format!("need_repaint page:{page_num} stage:{stage} {viewport:?}").into(),
        );

        let kern = self.client.clone().unwrap();
        let kern = kern.lock().unwrap();
        let page = &mut self.doc_view[page_num as usize];

        match stage {
            STAGE_LAYOUT => Ok({
                page.relayout(&self.canvas_backend)?;
                false
            }),
            STAGE_SVG => Ok(page.need_repaint_svg(viewport)),
            STAGE_SEMANTICS => Ok(page.need_repaint_semantics()),
            STAGE_PREPARE_CANVAS => {
                page.need_prepare_canvas(kern.module(), &mut self.canvas_backend)
            }
            STAGE_CANVAS => Ok(page.need_repaint_canvas(&self.canvas_backend)),
            _ => todo!(),
        }
    }

    pub fn repaint(
        &mut self,
        page_num: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        stage: u8,
    ) -> ZResult<JsValue> {
        // todo: overflow
        let viewport = Some(tiny_skia::Rect::from_xywh(x, y, w, h).unwrap());
        #[cfg(feature = "debug_recalc_stage")]
        web_sys::console::log_1(
            &format!("repaint page:{page_num} stage:{stage} {viewport:?}").into(),
        );

        let kern = self.client.clone().unwrap();
        let kern_lock = kern.lock().unwrap();

        let mut ctx = DomContext {
            tmpl: self.tmpl.clone(),
            stub: self.stub().clone(),
            module: kern_lock.module(),
            svg_backend: &mut self.svg_backend,
            canvas_backend: &mut self.canvas_backend,
            semantics_backend: &mut self.semantics_backend,
        };

        let page = &mut self.doc_view[page_num as usize];

        match stage {
            STAGE_SVG => {
                page.repaint_svg(&mut ctx)?;
            }
            STAGE_SEMANTICS => {
                page.repaint_semantics(&mut ctx)?;
            }
            STAGE_PREPARE_CANVAS => {
                if let Some(elem) = page.prepare_canvas(&mut ctx)? {
                    // explicit drop ctx to avoid async promise cature these variables
                    drop(ctx);
                    #[cfg(feature = "debug_repaint_canvas")]
                    web_sys::console::log_1(&format!("canvas state prepare: {page_num}").into());
                    let ppp = self.canvas_backend.pixel_per_pt;
                    let ts = tiny_skia::Transform::from_scale(ppp, ppp);
                    if let Some(fut) = elem.prepare(ts) {
                        #[allow(dropping_references)]
                        drop(self);
                        return Ok(wasm_bindgen_futures::future_to_promise(async move {
                            fut.await;

                            Ok(JsValue::UNDEFINED)
                        })
                        .into());
                    }
                    web_sys::console::log_1(
                        &format!("canvas state prepare done: {}", page_num).into(),
                    );
                }
            }
            STAGE_CANVAS => {
                // explicit drop ctx to avoid async promise cature these variables
                drop(ctx);
                let ppp = self.canvas_backend.pixel_per_pt;
                let page = &mut self.doc_view[page_num as usize];
                let fut = page.repaint_canvas(viewport, ppp)?;
                #[allow(dropping_references)]
                drop(self);
                return Ok(wasm_bindgen_futures::future_to_promise(async move {
                    fut.await;

                    Ok(JsValue::UNDEFINED)
                })
                .into());
            }
            _ => todo!(),
        }
        Ok(JsValue::UNDEFINED)
    }
}

impl IncrDomDocClient {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn set_client(&mut self, client: Arc<Mutex<IncrDocClient>>) {
        self.client = Some(client);
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
        let layouts = kern.doc.layouts[0].by_scalar();
        let Some(layouts) = layouts else {
            return;
        };
        let mut layout = layouts.first().unwrap();

        // web_sys::console::log_1(&format!("layouts: {:?}", layouts).into());

        if let Some(viewport) = viewport {
            // base scale = 2
            let base_cw = viewport.width();

            // web_sys::console::log_1(
            //     &format!("layouts base_cw: {:?} {:?}", viewport, base_cw).into(),
            // );

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

    fn retrack_pages(&mut self, kern: &mut IncrDocClient, elem: HookedElement) -> ZResult<bool> {
        // Checks out the current document layout.
        let pages = self.checkout_pages(kern);

        let mut dirty = self.doc_view.len() != pages.len();

        // Tracks the pages.
        if self.doc_view.len() > pages.len() {
            self.doc_view.truncate(pages.len());
        }
        for i in self.doc_view.len()..pages.len() {
            self.doc_view
                .push(DomPage::new_at(elem.hooked.clone(), self.tmpl.clone(), i));
        }
        for (page, data) in self.doc_view.iter_mut().zip(pages) {
            let sub_dirty = page.track_data(data);
            dirty = dirty || sub_dirty;
        }

        // Populates the glyphs to dom so that they get rendered
        let glyphs = self.svg_backend.populate_glyphs(kern);
        if let Some(glyphs) = glyphs {
            self.populate_glyphs
                .as_ref()
                .unwrap()
                .call1(&JsValue::NULL, &glyphs.into())
                .unwrap();
        }

        Ok(dirty)
    }

    pub fn reset(&mut self) {}

    pub fn create_element(&self, html: &str) -> Element {
        self.tmpl.create_element(html)
    }

    /// Render the document in the given window.
    pub async fn mount(&mut self, elem: HtmlElement) -> ZResult<()> {
        if let Some(old_elem) = self.elem.as_ref() {
            return Err(error_once!(
                "already mounted to",
                old_elem: format!("{:?}", old_elem.hooked)
            ));
        }

        // create typst-svg-resources by string
        self.elem = Some(HookedElement {
            hooked: elem.clone(),
        });
        self.svg_backend.reset();
        self.canvas_backend.reset();

        Ok(())
    }
}

pub struct DomContext<'m, 'a> {
    tmpl: XmlFactory,
    stub: Element,
    pub svg_backend: &'a mut SvgBackend,
    pub canvas_backend: &'a mut CanvasBackend,
    pub semantics_backend: &'a mut SemanticsBackend,
    pub module: &'m Module,
}

impl<'m> DomContext<'m, '_> {
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
