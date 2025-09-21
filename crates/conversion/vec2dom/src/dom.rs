use std::{
    future::Future,
    rc::Rc,
    sync::{Arc, Mutex},
};

use reflexo::hash::Fingerprint;
use reflexo::vector::ir::{self, Module, Page, Point, Scalar, Size, TextItem, TransformItem};
use reflexo::{error::prelude::*, ImmutStr};
use reflexo_vec2canvas::{CanvasElem, CanvasNode, CanvasOp, CanvasStateGuard};
use web_sys::{
    js_sys::Reflect,
    wasm_bindgen::{JsCast, JsValue},
    window, Element, HtmlCanvasElement, HtmlDivElement, HtmlElement, SvgGraphicsElement,
    SvgsvgElement,
};

use crate::{
    canvas_backend::CanvasBackend, factory::XmlFactory, svg_backend::FETCH_BBOX_TIMES, DomContext,
};

pub struct DomPage {
    /// Index
    idx: usize,
    /// The element to track.
    elem: HtmlElement,
    /// The canvas element to track.
    canvas: HtmlCanvasElement,
    /// The svg element to track.
    svg: SvgsvgElement,
    /// The semantics element to track.
    semantics: HtmlDivElement,
    /// The layout data, currently there is only a page in layout.
    layout_data: Option<Page>,
    /// The next page data
    dirty_layout: Option<Page>,
    /// The viewport.
    viewport: ir::Rect,
    /// The BBox of the page.
    bbox: ir::Rect,
    /// The realized element.
    pub realized: Rc<Mutex<Option<TypstPageElem>>>,
    /// The realized canvas element.
    realized_canvas: Option<CanvasNode>,
    /// The flushed semantics state.
    semantics_state: Option<(Page, bool)>,
    /// The flushed canvas state.
    canvas_state: Rc<Mutex<Option<CanvasRenderState>>>,
    /// Whether the page is visible.
    is_visible: bool,
    /// The group element.
    g: Element,
    /// The stub element.
    stub: Element,
}

impl Drop for DomPage {
    fn drop(&mut self) {
        self.elem.remove();
    }
}

struct CanvasRenderState {
    // (Page, f32)
    rendered: Page,
    ppp: f32,
    render_entire_page: bool,
}

impl DomPage {
    pub fn new_at(elem: HtmlElement, tmpl: XmlFactory, idx: usize) -> Self {
        // https://stackoverflow.com/questions/20242806/hole-in-overlay-with-css
        const TEMPLATE: &str = r#"<div class="typst-dom-page"><canvas class="typst-back-canvas" style="--reflexo-clip-lo-x: 0px; --reflexo-clip-lo-y: 0px; --reflexo-clip-hi-x: 0px; --reflexo-clip-hi-y: 0px; clip-path: polygon( evenodd, 0 0, 100% 0, 100% 100%, 0% 100%, 0 0, var(--reflexo-clip-lo-x) var(--reflexo-clip-lo-y), var(--reflexo-clip-hi-x) var(--reflexo-clip-lo-y), var(--reflexo-clip-hi-x) var(--reflexo-clip-hi-y), var(--reflexo-clip-lo-x) var(--reflexo-clip-hi-y), var(--reflexo-clip-lo-x) var(--reflexo-clip-lo-y))"></canvas><svg class="typst-svg-page" viewBox="0 0 0 0" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">
<g></g><stub></stub></svg><div class="typst-html-semantics"><div/></div>"#;

        let me = tmpl.create_element(TEMPLATE);
        me.set_attribute("data-index", &idx.to_string()).unwrap();
        let canvas: HtmlCanvasElement = me.first_element_child().unwrap().dyn_into().unwrap();
        let svg: SvgsvgElement = canvas.next_element_sibling().unwrap().dyn_into().unwrap();
        let semantics: HtmlDivElement = svg.next_element_sibling().unwrap().dyn_into().unwrap();
        let g = svg.first_element_child().unwrap();
        let stub = g.next_element_sibling().unwrap();
        g.remove();

        // window.bindSemantics
        let bind_semantics_handler = window().unwrap();
        let bind_semantics_handler =
            Reflect::get(&bind_semantics_handler, &"typstBindSemantics".into())
                .unwrap()
                .dyn_into::<web_sys::js_sys::Function>()
                .unwrap();
        bind_semantics_handler
            .call3(&JsValue::UNDEFINED, &me, &svg, &semantics)
            .unwrap();

        // for debug canvas
        // svg.style().set_property("visibility", "hidden").unwrap();
        // for debug svg
        // canvas.style().set_property("visibility", "hidden").unwrap();

        elem.append_child(&me).unwrap();

        // todo: default or none?
        let viewport = ir::Rect::default();
        let bbox = viewport;
        Self {
            is_visible: false,
            g,
            stub,
            idx,
            elem: me.dyn_into().unwrap(),
            canvas,
            svg,
            semantics,
            viewport,
            bbox,
            layout_data: None,
            dirty_layout: None,
            realized: Rc::new(Mutex::new(None)),
            realized_canvas: None,
            canvas_state: Rc::new(Mutex::new(None)),
            semantics_state: None,
        }
    }

    pub fn track_data(&mut self, data: &Page) -> bool {
        if self.layout_data.as_ref() == Some(data) {
            return false;
        }

        self.dirty_layout = Some(data.clone());
        true
    }

    fn pull_viewport(&mut self, viewport: Option<tiny_skia::Rect>) {
        self.viewport = viewport
            .and_then(|viewport| {
                tiny_skia::Rect::from_ltrb(
                    self.bbox.lo.x.0 - 1.,
                    viewport.top(),
                    self.bbox.hi.x.0 + 1.,
                    viewport.bottom(),
                )
            })
            .map(From::from)
            .unwrap_or(self.bbox);
        #[cfg(feature = "debug_repaint")]
        web_sys::console::log_2(
            &format!(
                "pull_viewport {idx} vp:{vp:?}, bbox {bbox:?}",
                idx = self.idx,
                vp = viewport,
                bbox = self.bbox,
            )
            .into(),
            &self.elem,
        );
    }

    fn change_svg_visibility(&mut self, should_visible: bool) {
        if should_visible != self.is_visible {
            #[cfg(feature = "debug_repaint")]
            web_sys::console::log_2(
                &format!(
                    "change_svg_visibility {idx} {should_visible}",
                    idx = self.idx,
                    should_visible = should_visible
                )
                .into(),
                &self.elem,
            );
            self.is_visible = should_visible;
            if should_visible {
                self.stub.replace_with_with_node_1(&self.g).unwrap();
            } else {
                self.g.replace_with_with_node_1(&self.stub).unwrap();
            }
        }
    }

    pub fn relayout(&mut self, ctx: &CanvasBackend) -> Result<()> {
        if let Some(data) = self.dirty_layout.take() {
            self.do_relayout(ctx, data)?
        }

        Ok(())
    }

    fn do_relayout(&mut self, ctx: &CanvasBackend, data: Page) -> Result<()> {
        #[cfg(feature = "debug_relayout")]
        web_sys::console::log_2(
            &format!("re-layout {idx} {data:?}", idx = self.idx).into(),
            &self.elem,
        );

        let prev_size = self.layout_data.as_ref().map(|d| d.size);

        if prev_size.map(|s| s != data.size).unwrap_or(true) {
            #[cfg(feature = "debug_relayout")]
            web_sys::console::log_2(
                &format!("resize {idx} {data:?}", idx = self.idx).into(),
                &self.elem,
            );

            // calculate the width and height of the svg
            // todo: don't update if individual not changed
            let w = data.size.x.0;
            let h = data.size.y.0;

            self.elem
                .set_attribute("data-width", &w.to_string())
                .unwrap();
            self.elem
                .set_attribute("data-height", &h.to_string())
                .unwrap();
            let style = self.elem.style();
            style
                .set_property("--data-page-width", &format!("{w:.3}px"))
                .unwrap();
            style
                .set_property("--data-page-height", &format!("{h:.3}px"))
                .unwrap();
            self.svg
                .set_attribute("viewBox", &format!("0 0 {w} {h}"))
                .unwrap();

            self.svg
                .set_attribute("data-width", &w.to_string())
                .unwrap();
            self.svg
                .set_attribute("data-height", &h.to_string())
                .unwrap();
            self.bbox = ir::Rect {
                lo: Point::default(),
                hi: data.size,
            };

            let ppp = ctx.pixel_per_pt;
            self.canvas.set_width((w * ppp) as u32);
            self.canvas.set_height((h * ppp) as u32);
            *self.canvas_state.lock().unwrap() = None;
            style
                .set_property(
                    "--data-canvas-scale",
                    &format!("{:.3}", 1. / ctx.pixel_per_pt),
                )
                .unwrap();
        }

        // todo: cache
        let prev_data = self.layout_data.clone();
        if prev_data.map(|d| d != data).unwrap_or(true) {
            #[cfg(feature = "debug_relayout")]
            web_sys::console::log_2(
                &format!("dirty layout detected {idx}", idx = self.idx).into(),
                &self.elem,
            );
            // self.change_svg_visibility(false);
            *self.realized.lock().unwrap() = None;
            *self.canvas_state.lock().unwrap() = None;
            self.semantics_state = None;
            self.realized_canvas = None;
            self.layout_data = Some(data);
        }

        Ok(())
    }

    pub fn need_repaint_svg(&mut self, viewport: Option<tiny_skia::Rect>) -> bool {
        self.pull_viewport(viewport);

        let should_visible = !self.bbox.intersect(&self.viewport).is_empty();

        if cfg!(feature = "debug_repaint_svg") {
            web_sys::console::log_1(
                &format!(
                    "need_repaint_svg({should_visible}) bbox:{bbox:?} view:{viewport:?}",
                    bbox = self.bbox,
                    viewport = self.viewport,
                )
                .into(),
            );
        }

        self.change_svg_visibility(should_visible);
        should_visible
    }

    pub fn repaint_svg(&mut self, ctx: &mut DomContext<'_, '_>) -> Result<()> {
        let should_visible = !self.bbox.intersect(&self.viewport).is_empty();

        if cfg!(feature = "debug_repaint") {
            web_sys::console::log_1(
                &format!(
                    "repaint_root({should_visible}) bbox:{bbox:?} view:{viewport:?}",
                    bbox = self.bbox,
                    viewport = self.viewport,
                )
                .into(),
            );
        }

        self.change_svg_visibility(should_visible);
        if !self.is_visible {
            return Ok(());
        }

        let mut realized = self.realized.lock().unwrap();

        if realized.is_none() {
            let data = self.layout_data.clone().unwrap();

            // todo incremental
            if self.realized_canvas.is_none() {
                self.realized_canvas = Some(ctx.canvas_backend.render_page(ctx.module, &data)?);
            }

            // Realize svg
            let g = ctx.svg_backend.render_page(ctx.module, &data, &self.g);
            let mut elem = TypstPageElem::from_elem(ctx, g, data);
            if elem.g.canvas.is_none() {
                elem.g.attach_canvas(self.realized_canvas.clone().unwrap());
            }

            *realized = Some(elem);

            // window.bindSvgDom
            let bind_dom_handler = window().unwrap();
            let bind_dom_handler = Reflect::get(&bind_dom_handler, &"typstBindSvgDom".into())
                .unwrap()
                .dyn_into::<web_sys::js_sys::Function>()
                .unwrap();
            bind_dom_handler
                .call2(&JsValue::UNDEFINED, &self.elem, &self.svg)
                .unwrap();
        }

        let ts = tiny_skia::Transform::identity();

        if let Some(attached) = realized.as_mut() {
            if cfg!(feature = "debug_repaint_svg") {
                web_sys::console::log_2(
                    &format!(
                        "repaint_start {idx} {vp:?}, bbox_query {fetch_times:?}",
                        idx = self.idx,
                        vp = self.viewport,
                        fetch_times = FETCH_BBOX_TIMES.load(std::sync::atomic::Ordering::SeqCst)
                    )
                    .into(),
                    &self.elem,
                );
            }

            attached.repaint_svg(ctx, ts, self.viewport);

            if cfg!(feature = "debug_repaint_svg") {
                web_sys::console::log_2(
                    &format!(
                        "repaint_end {idx} {vp:?}, bbox_query {fetch_times:?}",
                        idx = self.idx,
                        vp = self.viewport,
                        fetch_times = FETCH_BBOX_TIMES.load(std::sync::atomic::Ordering::SeqCst)
                    )
                    .into(),
                    &self.elem,
                );
            }
        }
        Ok(())
    }

    pub fn need_repaint_semantics(&mut self) -> bool {
        self.semantics_state.as_ref().is_none_or(|e| {
            let (data, layout_heavy) = e;
            e.0.content != data.content || (self.is_visible && !*layout_heavy)
        })
    }

    pub fn repaint_semantics(&mut self, ctx: &mut DomContext<'_, '_>) -> Result<()> {
        let init_semantics = self.semantics_state.as_ref().is_none_or(|e| {
            let (data, _layout_heavy) = e;
            e.0.content != data.content
        });

        if init_semantics {
            let do_heavy = self.is_visible;

            let data = self.layout_data.clone().unwrap();
            #[cfg(feature = "debug_repaint_semantics")]
            web_sys::console::log_1(
                &format!("init semantics({do_heavy}): {} {:?}", self.idx, data).into(),
            );

            let output = ctx.semantics_backend.render(ctx.module, &data, do_heavy);
            self.semantics.set_inner_html(&output);
            self.semantics_state = Some((data, do_heavy));
        }

        if !self.is_visible {
            return Ok(());
        }

        if self.semantics_state.as_ref().is_none_or(|e| e.1) {
            return Ok(());
        }

        let data = self.layout_data.clone().unwrap();
        web_sys::console::log_1(&format!("layout heavy semantics: {} {:?}", self.idx, data).into());

        let output = ctx.semantics_backend.render(ctx.module, &data, true);
        self.semantics.set_inner_html(&output);

        self.semantics_state.as_mut().unwrap().1 = true;
        Ok(())
    }

    pub fn need_prepare_canvas(&mut self, module: &Module, b: &mut CanvasBackend) -> Result<bool> {
        // already pulled
        // self.pull_viewport(viewport);

        let need_repaint = self.need_repaint_canvas(b);

        if need_repaint {
            let data = self.layout_data.clone().unwrap();

            // todo incremental
            if self.realized_canvas.is_none() {
                self.realized_canvas = Some(b.render_page(module, &data)?);
            }

            let ppp = b.pixel_per_pt;
            let ts = tiny_skia::Transform::from_scale(ppp, ppp);
            let should_load_resource = self.realized_canvas.as_ref().unwrap().prepare(ts).is_some();

            #[cfg(feature = "debug_repaint_canvas")]
            web_sys::console::log_1(
                &format!(
                    "canvas state prepare({}), should_load_resource:{}",
                    self.idx, should_load_resource
                )
                .into(),
            );

            return Ok(should_load_resource);
        }

        Ok(false)
    }

    pub fn prepare_canvas(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
    ) -> Result<Option<Arc<CanvasElem>>> {
        let need_repaint = self.need_repaint_canvas(ctx.canvas_backend);

        let res = if need_repaint {
            let data = self.layout_data.clone().unwrap();

            // todo incremental
            if self.realized_canvas.is_none() {
                self.realized_canvas = Some(ctx.canvas_backend.render_page(ctx.module, &data)?);
            }

            Some(self.realized_canvas.clone().unwrap())
        } else {
            None
        };

        Ok(res)
    }

    pub fn need_repaint_canvas(&mut self, b: &CanvasBackend) -> bool {
        // already pulled
        // self.pull_viewport(viewport);

        if self.is_visible {
            return true;
        }

        let state = self.layout_data.as_ref().unwrap();
        self.canvas_state.lock().unwrap().as_ref().is_none_or(|s| {
            !s.render_entire_page || s.rendered != *state || s.ppp != b.pixel_per_pt
        })
    }

    pub fn repaint_canvas(
        &mut self,
        _viewport: Option<tiny_skia::Rect>,
        ppp: f32,
    ) -> Result<impl Future<Output = ()>> {
        let render_entire_page = self.realized.lock().unwrap().is_none() || !self.is_visible;

        if let Some(attached) = self.realized.lock().unwrap().as_mut() {
            if attached.g.canvas.is_none() {
                // todo incremental
                attached.attach_canvas(self.realized_canvas.clone().unwrap());
            }
        };

        let canvas = self.canvas.clone();

        let _idx = self.idx;
        let state = self.layout_data.clone().unwrap();
        let canvas_state = self.canvas_state.clone();
        let canvas_elem = self.realized_canvas.clone().unwrap();
        let elem = self.realized.clone();

        #[cfg(feature = "debug_repaint_canvas")]
        web_sys::console::log_1(
            &format!("canvas check: {} {}", _idx, elem.lock().unwrap().is_some()).into(),
        );

        #[allow(clippy::await_holding_lock)]
        return Ok(async move {
            let canvas_ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            let mut elem = elem.lock().unwrap();

            #[cfg(feature = "debug_repaint_canvas")]
            web_sys::console::log_1(&format!("canvas render: {_idx} {_viewport:?}").into());

            'render_canvas: {
                let _global_guard = CanvasStateGuard::new(&canvas_ctx);

                if canvas_state
                    .lock()
                    .unwrap()
                    .as_ref()
                    .is_some_and(|s| s.rendered == state && s.ppp == ppp)
                {
                    break 'render_canvas;
                }
                #[cfg(feature = "debug_repaint_canvas")]
                web_sys::console::log_1(
                    &format!(
                        "canvas state changed, render all: {} {}",
                        idx,
                        elem.is_none()
                    )
                    .into(),
                );

                // todo: memorize canvas fill
                canvas_ctx.clear_rect(
                    0.,
                    0.,
                    (state.size.x.0 * ppp) as f64,
                    (state.size.y.0 * ppp) as f64,
                );

                *canvas_state.lock().unwrap() = Some(CanvasRenderState {
                    rendered: state.clone(),
                    ppp,
                    render_entire_page: true,
                });

                let ts = tiny_skia::Transform::from_scale(ppp, ppp);
                canvas_elem.realize(ts, &canvas_ctx).await;
            }

            let mut clip_rect = None;
            'clip_canvas: {
                if render_entire_page {
                    break 'clip_canvas;
                } else {
                    let all_painted = canvas_state
                        .clone()
                        .lock()
                        .unwrap()
                        .as_ref()
                        .is_some_and(|e| e.render_entire_page);

                    let Some(elem) = elem.as_mut() else {
                        panic!("realized is none for partial canvas render");
                    };

                    if all_painted {
                        elem.mark_painted();
                    }

                    let ts = tiny_skia::Transform::identity();
                    let Some(damage_rect) = elem.get_damage_rect(ts) else {
                        break 'clip_canvas;
                    };

                    #[cfg(feature = "debug_repaint_canvas")]
                    web_sys::console::log_1(
                        &format!("canvas partial render: {idx} {damage_rect:?}").into(),
                    );

                    // let x = damage_rect.lo.x.0;
                    let y = damage_rect.lo.y.0;
                    // let w = damage_rect.width().0;
                    let h = damage_rect.height().0;

                    let data_size = state.size;
                    // let dx = data_size.x.0.max(1.);
                    let dy = data_size.y.0.max(1.);

                    let x_ratio = 0.;
                    let y_ratio = y / dy * 100.;
                    let w_ratio = 100.;
                    let h_ratio = h / dy * 100.;

                    clip_rect = Some((x_ratio, y_ratio, w_ratio, h_ratio));

                    let _ = DomPage::repaint_canvas;
                }
            }

            let clip_key = format!("{clip_rect:?}");

            // data-clip-rect-state
            const CLIP_KEY: &str = "data-clip-rect-state";
            let clip_state = canvas.get_attribute(CLIP_KEY).unwrap_or_default();

            if clip_state != clip_key {
                let _ = canvas.set_attribute(CLIP_KEY, &clip_key);

                if let Some((x, y, w, h)) = clip_rect {
                    let modify = |p, x| canvas.style().set_property(p, &format!("{x:.3}%"));
                    let _ = modify("--reflexo-clip-lo-x", x);
                    let _ = modify("--reflexo-clip-lo-y", y);
                    let _ = modify("--reflexo-clip-hi-x", x + w);
                    let _ = modify("--reflexo-clip-hi-y", y + h);
                } else {
                    let modify = |p| canvas.style().remove_property(p);
                    let _ = modify("--reflexo-clip-lo-x");
                    let _ = modify("--reflexo-clip-lo-y");
                    let _ = modify("--reflexo-clip-hi-x");
                    let _ = modify("--reflexo-clip-hi-y");
                }
            }
        });
    }
}

#[derive(Debug)]
pub struct TypstPageElem {
    pub stub: Element,
    pub g: TypstElem,
    pub clip_paths: Element,
    pub style_defs: Element,
}

impl TypstPageElem {
    pub fn from_elem(ctx: &mut DomContext<'_, '_>, g: Element, data: Page) -> Self {
        let g = g.first_element_child().unwrap();
        let stub = ctx.create_stub();
        let clip_paths = g.next_element_sibling().unwrap();
        let style_defs = clip_paths.next_element_sibling().unwrap();
        let attached = Self::attach_html(ctx, g.clone().dyn_into().unwrap(), data.content);

        Self {
            g: attached,
            stub,
            clip_paths,
            style_defs,
        }
    }
}

#[derive(Debug)]
pub enum TypstDomExtra {
    Group(GroupElem),
    Item(TransformElem),
    Label(LabelElem),
    Image(ImageElem),
    Text(TextElem),
    Path(PathElem),
    RawHtml(HtmlElem),
    Link(LinkElem),
    ContentHint(ContentHintElem),
}

#[derive(Debug)]
pub struct TypstElem {
    pub is_svg_visible: bool,
    pub is_canvas_painted: bool,
    pub f: Fingerprint,
    pub extra: TypstDomExtra,

    /// Stub backend specific data
    pub stub: Element,
    /// Svg backend specific data
    pub g: SvgGraphicsElement,
    /// Canvas backend specific data
    pub canvas: Option<CanvasNode>,
}

#[derive(Debug)]
pub struct ImageElem {
    pub size: Size,
}

#[derive(Debug)]
pub struct TextElem {
    pub upem: Scalar,
    pub meta: TextItem,
}
#[derive(Debug)]
pub struct HtmlElem {}
#[derive(Debug)]
pub struct PathElem {}
#[derive(Debug)]
pub struct LinkElem {}
#[derive(Debug)]
pub struct ContentHintElem {
    pub hint: char,
}

#[derive(Debug)]
pub struct TransformElem {
    pub trans: TransformItem,
    pub child: Box<TypstElem>,
}

#[derive(Debug)]
pub struct LabelElem {
    pub label: ImmutStr,
    pub child: Box<TypstElem>,
}

#[derive(Debug)]
pub struct GroupElem {
    pub children: Vec<(Point, TypstElem)>,
}
