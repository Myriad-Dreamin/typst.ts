use typst_ts_core::{
    error::prelude::*,
    hash::Fingerprint,
    vector::ir::{Page, Point, Scalar, Size, TextItem, TransformItem},
};
use web_sys::{wasm_bindgen::JsCast, Element, HtmlElement, SvgGraphicsElement, SvgsvgElement};

use crate::{factory::XmlFactory, svg_backend::FETCH_BBOX_TIMES, DomContext};

pub struct DomPage {
    /// Index
    idx: usize,
    /// The element to track.
    elem: HtmlElement,
    /// The canvas element to track.
    canvas: Element,
    /// The svg element to track.
    svg: SvgsvgElement,
    /// The page data
    data: Option<Page>,
    /// The page data
    next_data: Option<Page>,
    /// The viewport.
    viewport: tiny_skia::Rect,
    /// The BBox of the page.
    bbox: tiny_skia::Rect,
    attached: Option<TypstPageElem>,
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

impl DomPage {
    pub fn new_at(elem: HtmlElement, tmpl: XmlFactory, idx: usize) -> Self {
        const TEMPLATE: &str = r#"<div class="typst-dom-page"><canvas class="typst-back-canvas"></canvas><svg class="typst-svg-page" viewBox="0 0 0 0" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">
<g></g><stub></stub></svg></div>"#;

        let me = tmpl.create_element(TEMPLATE);
        me.set_attribute("data-index", &idx.to_string()).unwrap();
        let canvas = me.first_element_child().unwrap();
        let svg: SvgsvgElement = canvas.next_element_sibling().unwrap().dyn_into().unwrap();
        let g = svg.first_element_child().unwrap();
        let stub = g.next_element_sibling().unwrap();
        g.remove();

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
            viewport,
            bbox,
            data: None,
            next_data: None,
            attached: None,
        }
    }

    pub fn track_data(&mut self, data: &Page) {
        if self.data.as_ref().map_or(false, |d| d == data) {
            return;
        }

        let prev_size = self.data.as_ref().map(|d| d.size);

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
        }

        self.next_data = Some(data.clone());
    }

    fn attach_dom(&mut self, ctx: &mut DomContext<'_, '_>, g: Element, data: Page) -> ZResult<()> {
        web_sys::console::log_2(
            &format!("attach {idx} {a:?}", idx = self.idx, a = self.attached).into(),
            &g,
        );
        self.attached = Some(TypstPageElem::from_elem(ctx, g, data));

        Ok(())
    }

    async fn do_recalculate(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
        data: Page,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<()> {
        self.reflow(ctx, data.clone()).await?;

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
        let viewport = viewport.unwrap_or_else(|| {
            tiny_skia::Rect::from_xywh(0., 0., data.size.x.0, data.size.y.0).unwrap()
        });
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
            return Ok(());
        };
        self.viewport = viewport;

        self.repaint(ctx).await?;

        Ok(())
    }

    /// Triggle a recalculation.
    pub async fn recalculate(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<()> {
        if let Some(data) = self.next_data.take() {
            self.data = Some(data);
        }

        if let Some(data) = self.data.clone() {
            self.do_recalculate(ctx, data, viewport).await?;
        }
        Ok(())
    }

    async fn reflow(&self, _ctx: &mut DomContext<'_, '_>, _data: Page) -> ZResult<()> {
        web_sys::console::log_2(&format!("reflow {idx}", idx = self.idx).into(), &self.elem);

        Ok(())
    }

    async fn repaint(&mut self, ctx: &mut DomContext<'_, '_>) -> ZResult<()> {
        let should_visible = self.bbox.intersect(&self.viewport).is_some();

        web_sys::console::log_1(
            &format!(
                "repaint_root({should_visible}) bbox:{bbox:?} view:{viewport:?}",
                bbox = self.bbox,
                viewport = self.viewport,
            )
            .into(),
        );

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

        if self.attached.is_none() {
            let data = self.data.clone().unwrap();
            let g = ctx.svg_backend.render_page(ctx.module, &data, &self.g);
            self.attach_dom(ctx, g, data)?;
        }

        let ts = tiny_skia::Transform::identity();

        web_sys::console::log_2(
            &format!(
                "repaint {idx} {vp:?}, bbox_query {fetch_times:?}",
                idx = self.idx,
                vp = self.viewport,
                fetch_times = FETCH_BBOX_TIMES.load(std::sync::atomic::Ordering::SeqCst)
            )
            .into(),
            &self.elem,
        );

        if let Some(attached) = &mut self.attached {
            attached.repaint(ctx, ts, self.viewport);
        }

        web_sys::console::log_2(
            &format!(
                "reflow_end {idx} {vp:?}, bbox_query {fetch_times:?}",
                idx = self.idx,
                vp = self.viewport,
                fetch_times = FETCH_BBOX_TIMES.load(std::sync::atomic::Ordering::SeqCst)
            )
            .into(),
            &self.elem,
        );
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
    pub is_visible: bool,
    pub browser_bbox_unchecked: bool,
    pub stub: Element,
    pub g: SvgGraphicsElement,
    pub f: Fingerprint,
    pub estimated_bbox: Option<tiny_skia::Rect>,
    pub bbox: Option<tiny_skia::Rect>,
    pub extra: TypstDomExtra,
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
