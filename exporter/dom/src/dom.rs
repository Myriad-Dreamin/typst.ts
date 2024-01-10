use typst_ts_canvas_exporter::{CanvasAction, CanvasNode, CanvasStateGuard};
use typst_ts_core::{
    error::prelude::*,
    hash::Fingerprint,
    vector::ir::{Page, Point, Scalar, Size, TextItem, TransformItem},
};
use web_sys::{
    wasm_bindgen::JsCast, Element, HtmlCanvasElement, HtmlDivElement, HtmlElement,
    SvgGraphicsElement, SvgsvgElement,
};

use crate::{
    factory::XmlFactory, semantics_backend::SemanticsBackend, svg_backend::FETCH_BBOX_TIMES,
    DomContext,
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
    viewport: tiny_skia::Rect,
    /// The BBox of the page.
    bbox: tiny_skia::Rect,
    /// The realized element.
    realized: Option<TypstPageElem>,
    /// The realized canvas element.
    realized_canvas: Option<CanvasNode>,
    /// The flushed semantics state.
    semantics_state: Option<(Page, bool)>,
    /// The flushed canvas state.
    canvas_state: Option<(Page, f32)>,
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

static A_WIDTH: once_cell::sync::OnceCell<f32> = once_cell::sync::OnceCell::new();

impl DomPage {
    pub fn new_at(elem: HtmlElement, tmpl: XmlFactory, idx: usize) -> Self {
        const TEMPLATE: &str = r#"<div class="typst-dom-page"><canvas class="typst-back-canvas"></canvas><svg class="typst-svg-page" viewBox="0 0 0 0" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">
<g></g><stub></stub></svg><div class="typst-html-semantics"><div/></div>"#;

        let me = tmpl.create_element(TEMPLATE);
        me.set_attribute("data-index", &idx.to_string()).unwrap();
        let canvas: HtmlCanvasElement = me.first_element_child().unwrap().dyn_into().unwrap();
        let svg: SvgsvgElement = canvas.next_element_sibling().unwrap().dyn_into().unwrap();
        let semantics: HtmlDivElement = svg.next_element_sibling().unwrap().dyn_into().unwrap();
        let g = svg.first_element_child().unwrap();
        let stub = g.next_element_sibling().unwrap();
        g.remove();

        // for debug canvas
        // svg.style().set_property("visibility", "hidden").unwrap();

        elem.append_child(&me).unwrap();

        let viewport = tiny_skia::Rect::from_xywh(0., 0., 0., 0.).unwrap();
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
            realized: None,
            realized_canvas: None,
            canvas_state: None,
            semantics_state: None,
        }
    }

    pub fn track_data(&mut self, data: &Page) {
        if self.layout_data.as_ref().map_or(false, |d| d == data) {
            return;
        }

        self.dirty_layout = Some(data.clone());
    }

    /// Triggle a recalculation.
    pub async fn recalculate(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<()> {
        // Update flow if needed.
        let dirty_layout = if let Some(data) = self.dirty_layout.take() {
            self.relayout(ctx, data)?
        } else {
            false
        };
        assert!(self.dirty_layout.is_none());

        let dirty_viewport = self.pull_viewport(viewport)?;

        // If there is no layout, skip the next stages.
        // If there is no paint needed, as well.
        if self.layout_data.is_none() || !(dirty_viewport || dirty_layout) {
            return Ok(());
        }

        // Repaint a page as svg.
        self.repaint_svg(ctx)?;

        // Repaint a page as semantics.
        self.repaint_semantics(ctx)?;

        // Repaint a page as canvas.
        self.repaint_canvas(ctx).await?;

        Ok(())
    }

    fn relayout(&mut self, ctx: &mut DomContext<'_, '_>, data: Page) -> ZResult<bool> {
        web_sys::console::log_2(
            &format!("re-layout {idx}", idx = self.idx).into(),
            &self.elem,
        );

        let prev_size = self.layout_data.as_ref().map(|d| d.size);

        if prev_size.map(|s| s != data.size).unwrap_or(true) {
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
                .set_property("--data-page-width", &format!("{:.3}px", w))
                .unwrap();
            style
                .set_property("--data-page-height", &format!("{:.3}px", h))
                .unwrap();
            self.svg
                .set_attribute(
                    "viewBox",
                    &format!("0 0 {width} {height}", width = w, height = h),
                )
                .unwrap();
            self.bbox = tiny_skia::Rect::from_xywh(0., 0., data.size.x.0, data.size.y.0).unwrap();

            let ppp = ctx.canvas_backend.pixel_per_pt;
            self.canvas.set_width((w * ppp) as u32);
            self.canvas.set_height((h * ppp) as u32);
            self.canvas_state = None;
            style
                .set_property("--data-canvas-scale", &format!("{:.3}", 1. / 3.))
                .unwrap();
        }

        self.layout_data = Some(data);
        Ok(true)
    }

    fn pull_viewport(&mut self, viewport: Option<tiny_skia::Rect>) -> ZResult<bool> {
        let ctm = self.svg.get_ctm().unwrap();
        // todo: cause a real reflow
        let cr = self.elem.get_bounding_client_rect();
        let ts = tiny_skia::Transform::from_row(
            ctm.a(),
            ctm.b(),
            ctm.c(),
            ctm.d(),
            ctm.e() - cr.left() as f32,
            ctm.f() - cr.top() as f32,
        );
        let viewport = viewport.unwrap_or(self.bbox);
        let Some(viewport) = viewport.transform(ts) else {
            web_sys::console::warn_2(
                &format!(
                    "viewport is empty: {vp:?}, ts: {ts:?}, cr: {cr:?}",
                    vp = viewport,
                    ts = ts,
                    cr = cr,
                )
                .into(),
                &self.elem,
            );
            return Ok(false);
        };

        if self.viewport == viewport {
            Ok(false)
        } else {
            self.viewport = viewport;
            Ok(true)
        }
    }

    fn repaint_svg(&mut self, ctx: &mut DomContext<'_, '_>) -> ZResult<()> {
        let should_visible = self.bbox.intersect(&self.viewport).is_some();

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

        if should_visible != self.is_visible {
            self.is_visible = should_visible;
            if should_visible {
                self.stub.replace_with_with_node_1(&self.g).unwrap();
            } else {
                self.g.replace_with_with_node_1(&self.stub).unwrap();
                return Ok(());
            }
        } else if !should_visible {
            return Ok(());
        }

        if self.realized.is_none() {
            let data = self.layout_data.clone().unwrap();

            // Realize svg
            let g = ctx.svg_backend.render_page(ctx.module, &data, &self.g);
            self.realized = Some(TypstPageElem::from_elem(ctx, g, data));
        }

        let ts = tiny_skia::Transform::identity();

        if let Some(attached) = &mut self.realized {
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

    fn repaint_semantics(&mut self, ctx: &mut DomContext<'_, '_>) -> ZResult<()> {
        let init_semantics = self.semantics_state.as_ref().map_or(true, |e| {
            let (data, _layout_heavy) = e;
            e.0.content != data.content
        });

        let a_width: f32 = *A_WIDTH.get_or_init(|| {
            let ctx = self
                .canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            let _g = CanvasStateGuard::new(&ctx);
            ctx.set_font("128px monospace");
            let a_width = ctx.measure_text("A").unwrap().width();

            (a_width / 128.) as f32
        });

        if init_semantics {
            let do_heavy = self.is_visible;

            web_sys::console::log_1(&format!("init semantics({do_heavy}): {}", self.idx).into());

            let data = self.layout_data.clone().unwrap();
            let mut output = vec![];
            let mut t = SemanticsBackend::new(do_heavy, a_width, data.size.x.0);
            let ts = tiny_skia::Transform::identity();
            t.render_semantics(ctx.module, ts, data.content, &mut output);
            self.semantics.set_inner_html(&output.concat());
            self.semantics_state = Some((data, do_heavy));
        }

        if !self.is_visible {
            return Ok(());
        }

        if self.semantics_state.as_ref().map_or(true, |e| e.1) {
            return Ok(());
        }

        web_sys::console::log_1(&format!("layout heavy semantics: {}", self.idx).into());

        let data = self.layout_data.clone().unwrap();
        let mut output = vec![];
        let mut t = SemanticsBackend::new(true, a_width, data.size.x.0);
        let ts = tiny_skia::Transform::identity();
        t.render_semantics(ctx.module, ts, data.content, &mut output);
        self.semantics.set_inner_html(&output.concat());

        self.semantics_state.as_mut().unwrap().1 = true;
        Ok(())
    }

    async fn repaint_canvas(&mut self, ctx: &mut DomContext<'_, '_>) -> ZResult<()> {
        let render_entire_page = self.realized.is_none() || !self.is_visible;

        // todo incremental
        if self.realized_canvas.is_none() {
            let data = self.layout_data.as_ref().unwrap();
            self.realized_canvas = Some(ctx.canvas_backend.render_page(ctx.module, data)?);
        }

        if let Some(attached) = &mut self.realized {
            if attached.g.canvas.is_none() {
                attached.attach_canvas(self.realized_canvas.clone().unwrap());
            }
        };

        let canvas_ctx = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let ppp = ctx.canvas_backend.pixel_per_pt;
        let ts = tiny_skia::Transform::from_scale(ppp, ppp);

        'render_canvas: {
            let _global_guard = CanvasStateGuard::new(&canvas_ctx);

            let state = self.layout_data.clone().unwrap();
            if render_entire_page {
                if self
                    .canvas_state
                    .as_ref()
                    .map_or(false, |(x, y)| *x == state && *y == ppp)
                {
                    break 'render_canvas;
                }
                #[cfg(feature = "debug_repaint_canvas")]
                web_sys::console::log_1(
                    &format!("canvas state changed, render all: {}", self.idx).into(),
                );

                let canvas = self.realized_canvas.as_ref().unwrap();

                canvas_ctx.clear_rect(
                    0.,
                    0.,
                    (state.size.x.0 * ppp) as f64,
                    (state.size.y.0 * ppp) as f64,
                );

                self.canvas_state = Some((state, ppp));

                canvas.realize(ts, &canvas_ctx).await;
            } else {
                #[cfg(feature = "debug_repaint_canvas")]
                web_sys::console::log_1(&format!("canvas partial render: {}", self.idx).into());

                let Some(attached) = &mut self.realized else {
                    panic!("realized is none for partial canvas render");
                };

                // todo: memorize canvas fill
                canvas_ctx.clear_rect(
                    0.,
                    0.,
                    (state.size.x.0 * ppp) as f64,
                    (state.size.y.0 * ppp) as f64,
                );
                self.canvas_state = None;
                attached.repaint_canvas(ctx, ts, &canvas_ctx).await;
            }
        }

        Ok(())
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
        let attached = Self::attach_svg(ctx, g.clone().dyn_into().unwrap(), data.content);

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
    Image(ImageElem),
    Text(TextElem),
    Path(PathElem),
    Link(LinkElem),
    ContentHint(ContentHintElem),
}

#[derive(Debug)]
pub(crate) struct TypstElem {
    pub is_svg_visible: bool,
    pub browser_bbox_unchecked: bool,
    pub estimated_bbox: Option<tiny_skia::Rect>,
    pub bbox: Option<Box<tiny_skia::Rect>>,
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
pub struct GroupElem {
    pub children: Vec<(Point, TypstElem)>,
    pub size: Option<Size>,
}
