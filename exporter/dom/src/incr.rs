use core::fmt;
use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use tiny_skia::Transform;
use typst_ts_core::{
    error::prelude::*,
    hash::Fingerprint,
    vector::{
        incr::{IncrDocClient, IncrDocServer},
        ir::{FontItem, FontRef, LayoutRegionNode, Module, Page, VecItem},
    },
};
use wasm_bindgen_futures::spawn_local;
use web_sys::{wasm_bindgen::JsCast, Element, HtmlElement};

use crate::{
    canvas_backend::CanvasBackend, dom::DomPage, factory::XmlFactory, svg_backend::SvgBackend,
};

pub type IncrDOMDocServer = IncrDocServer;

// const NULL_PAGE: Fingerprint = Fingerprint::from_u128(1);

// struct DoRender ();

pub struct RenderTaskThis<'a> {
    pub cli_self: &'a mut IncrDomDocClient,
    pub cli_kern: &'a mut IncrDocClient,
}

pub enum RenderFuture {
    Generic(Box<dyn FnOnce(RenderTaskThis<'_>) -> RenderTask>),
    RecalcPage(usize, Transform),
}

impl fmt::Debug for RenderFuture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderFuture::Generic(_) => f.debug_struct("Generic").finish(),
            RenderFuture::RecalcPage(i, ts) => f
                .debug_struct("RecalcPage")
                .field("i", i)
                .field("ts", ts)
                .finish(),
        }
    }
}

#[derive(Debug)]
pub struct RenderMicroTask {
    pub then: RenderFuture,
    pub cancel: Option<RenderFuture>,
}

#[derive(Default, Clone)]
pub struct RenderTask(Arc<Mutex<Vec<RenderMicroTask>>>);

impl RenderTask {
    pub fn is_finished(&self) -> bool {
        self.0.lock().unwrap().is_empty()
    }

    pub fn transact(&self) -> MutexGuard<'_, Vec<RenderMicroTask>> {
        self.0.lock().unwrap()
    }
}

#[derive(Default)]
enum TrackMode {
    #[default]
    Document,
}

#[derive(Default, PartialEq, Eq)]
pub struct RecalcMode {
    is_responsive: bool,
    is_reschedule: bool,
}

impl RecalcMode {
    fn resp(r: bool) -> Self {
        Self {
            is_responsive: r,
            ..Default::default()
        }
    }

    fn sche(r: bool) -> Self {
        Self {
            is_reschedule: true,
            is_responsive: r,
        }
    }
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
    Recalc(RecalcMode, RenderTask),
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
    /// Shared render task
    task: RenderTask,

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

    pub fn reset(&mut self) {}

    /// Render the document in the given window.
    pub async fn mount(
        &mut self,
        kern: &mut IncrDocClient,
        elem: HtmlElement,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<RenderTask> {
        self.batch_dom_events(
            kern,
            vec![
                DOMChanges::Mount(elem),
                DOMChanges::Viewport(viewport),
                DOMChanges::Recalc(RecalcMode::resp(false), self.task.clone()),
            ],
        )
        .await
    }

    /// Render the document in the given window.
    pub async fn rerender(
        &mut self,
        kern: &mut IncrDocClient,
        viewport: Option<tiny_skia::Rect>,
        is_responsive: bool,
    ) -> ZResult<RenderTask> {
        self.batch_dom_events(
            kern,
            vec![
                DOMChanges::Viewport(viewport),
                DOMChanges::Recalc(RecalcMode::resp(is_responsive), self.task.clone()),
            ],
        )
        .await
    }

    /// Render the document in the given window.
    pub async fn reschedule(
        &mut self,
        kern: &mut IncrDocClient,
        render_task: RenderTask,
        is_responsive: bool,
    ) -> ZResult<RenderTask> {
        self.batch_dom_events(
            kern,
            vec![DOMChanges::Recalc(
                RecalcMode::sche(is_responsive),
                render_task,
            )],
        )
        .await
    }

    fn cancel_rendering(&self, _kern: &mut IncrDocClient) {
        let mut tasks = self.task.transact();
        if !tasks.is_empty() {
            // todo: call cancel
            tasks.clear();
        }
    }
}

impl IncrDomDocClient {
    /// Emit a batch of changes.
    async fn batch_dom_events(
        &mut self,
        kern: &mut IncrDocClient,
        changes: impl IntoIterator<Item = DOMChanges>,
    ) -> ZResult<RenderTask> {
        for change in changes {
            match change {
                DOMChanges::Unmount(elem) => {
                    self.cancel_rendering(kern);

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
                DOMChanges::Recalc(mode, task) => {
                    if let Some(elem) = &self.elem {
                        return self.recalculate(kern, elem.clone(), mode, task).await;
                    } else {
                        self.cancel_rendering(kern);
                    }
                }
            }
        }

        Ok(self.task.clone())
    }

    #[allow(clippy::await_holding_lock)]
    async fn recalculate(
        &mut self,
        kern: &mut IncrDocClient,
        elem: HookedElement,
        checkout_mode: RecalcMode,
        task: RenderTask,
    ) -> ZResult<RenderTask> {
        let mut tasks = task.transact();

        let RecalcMode {
            is_responsive,
            is_reschedule,
        } = checkout_mode;

        web_sys::console::log_1(
            &format!(
                "dom task({is_responsive},{is_reschedule}) recalculate: {:?}",
                tasks
            )
            .into(),
        );

        if is_reschedule && tasks.is_empty() {
            web_sys::console::log_1(&format!("bad dom task: {:?}", tasks).into());

            drop(tasks);
            return Ok(task);
        }

        if tasks.len() > 1 {
            todo!()
        }

        let checkout_mode = if is_responsive {
            CheckoutMode::Responsive
        } else {
            CheckoutMode::Full
        };

        let ret = if !tasks.is_empty() {
            let micro_task = tasks.pop().unwrap();
            web_sys::console::log_1(&format!("exec dom micro_task: {:?}", micro_task).into());
            match micro_task.then {
                RenderFuture::Generic(f) => {
                    return Ok(f(RenderTaskThis {
                        cli_self: self,
                        cli_kern: kern,
                    }))
                }
                RenderFuture::RecalcPage(i, ts) => {
                    let mut ctx = DomContext {
                        tmpl: self.tmpl.clone(),
                        stub: self.stub().clone(),
                        module: kern.module(),
                        svg_backend: &mut self.svg_backend,
                        canvas_backend: &mut self.canvas_backend,
                        checkout_mode,
                    };

                    let page = &mut self.doc_view[i];

                    let unfinished = page
                        .recalculate(
                            &mut ctx,
                            self.viewport
                                .and_then(|e: tiny_skia_path::Rect| e.transform(ts)),
                        )
                        .await?;

                    if let Some(unfinished) = unfinished {
                        let idx = i;
                        spawn_local(async move {
                            web_sys::console::log_1(&format!("dom task unfinished: {idx}").into());
                            unfinished.await;
                            web_sys::console::log_1(
                                &format!("dom task post ready, todo reschedule: {idx}").into(),
                            );
                        });
                    }

                    let i = i + 1;

                    let ts = ts.post_translate(0.0, page.uncommitted_height());
                    if i < self.doc_view.len() {
                        vec![RenderMicroTask {
                            then: RenderFuture::RecalcPage(i, ts),
                            cancel: None,
                        }]
                    } else {
                        Vec::default()
                    }
                }
            }
        } else {
            let dirty = match self.track_mode {
                TrackMode::Document => self.retrack_pages(kern, elem).await?,
                /* TrackMode::Pages => todo!(), */
            };

            if dirty && !self.doc_view.is_empty() {
                vec![RenderMicroTask {
                    then: RenderFuture::RecalcPage(0, Transform::identity()),
                    cancel: None,
                }]
            } else {
                Vec::default()
            }
        };

        if ret.is_empty() {
            tasks.clear();
        } else {
            *tasks = ret;
        }

        drop(tasks);
        Ok(task)
    }

    async fn retrack_pages(
        &mut self,
        kern: &mut IncrDocClient,
        elem: HookedElement,
    ) -> ZResult<bool> {
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
        self.svg_backend.populate_glyphs(kern, &elem);

        Ok(dirty)
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
