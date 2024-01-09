#![allow(dead_code)]

use std::{ops::Deref, sync::atomic::AtomicBool};

use typst_ts_core::{
    error::prelude::*,
    hash::Fingerprint,
    vector::{
        incr::{IncrDocClient, IncrDocServer},
        ir::TextItem,
        vm::{RenderState, RenderVm},
    },
};
use typst_ts_svg_exporter::{
    ir::{
        self, FontItem, FontRef, LayoutRegionNode, Page, Point, Scalar, Size, TransformItem,
        TransformedRef, VecItem,
    },
    Module, SvgExporter, SvgTask, SvgText,
};
use web_sys::{
    wasm_bindgen::JsCast, Element, HtmlElement, HtmlTemplateElement, SvgGraphicsElement,
    SvgsvgElement,
};

pub type IncrDOMDocServer = IncrDocServer;

/// The feature set which is used for exporting incremental rendered svg.
struct IncrementalSvgExportFeature;

impl typst_ts_svg_exporter::ExportFeature for IncrementalSvgExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    const USE_STABLE_GLYPH_ID: bool = true;
    const SHOULD_RASTERIZE_TEXT: bool = false;
    const WITH_BUILTIN_CSS: bool = false;
    const WITH_RESPONSIVE_JS: bool = false;
    const AWARE_HTML_ENTITY: bool = true;
}

type SvgBackend = SvgExporter<IncrementalSvgExportFeature>;
type SvgBackendTask = SvgTask<'static, IncrementalSvgExportFeature>;

// const NULL_PAGE: Fingerprint = Fingerprint::from_u128(1);

#[derive(Default)]
enum TrackMode {
    #[default]
    Document,
}

pub enum DOMChanges {
    /// Change the element to track.
    Unmount(HtmlElement),
    /// Change the element to track.
    Mount(HtmlElement),
    /// Change viewport.
    Viewport(Option<tiny_skia::Rect>),
}

#[derive(Debug, Clone)]
struct HookedElement {
    hooked: HtmlElement,
    resource_header: Element,
}

/// maintains the state of the incremental rendering at client side
#[derive(Default)]
pub struct IncrDomDocClient {
    tmpl: once_cell::sync::OnceCell<HtmlTemplateElement>,
    stub: once_cell::sync::OnceCell<Element>,
    /// Expected exact state of the current DOM.
    /// Initially it is None meaning no any page is rendered.
    doc_view: Vec<DomPage>,
    /// Track mode.
    track_mode: TrackMode,
    /// Glyphs that has already committed to the DOM.
    /// Assmuing glyph_window = N, then `self.doc.module.glyphs[..N]` are
    /// committed.
    pub glyph_window: usize,
    /// The element to track.
    elem: Option<HookedElement>,
    /// The viewport.
    viewport: Option<tiny_skia::Rect>,
}

impl IncrDomDocClient {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    fn tmpl(&self) -> &HtmlTemplateElement {
        self.tmpl.get_or_init(|| {
            let tmpl = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("template")
                .unwrap();
            tmpl.dyn_into().unwrap()
        })
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

    fn create_element(&self, html: &str) -> Element {
        let tmpl = self.tmpl();
        tmpl.set_inner_html(html);
        tmpl.content().first_element_child().unwrap()
    }

    pub fn reset(&mut self) {}

    /// Render the document in the given window.
    pub async fn mount(&mut self, kern: &mut IncrDocClient, elem: HtmlElement) -> ZResult<()> {
        self.batch_dom_events(kern, vec![DOMChanges::Mount(elem)])
            .await
    }

    /// Render the document in the given window.
    pub async fn rerender(
        &mut self,
        kern: &mut IncrDocClient,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<()> {
        self.batch_dom_events(kern, vec![DOMChanges::Viewport(viewport)])
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
                    self.glyph_window = 0;
                }
                DOMChanges::Viewport(viewport) => {
                    self.viewport = viewport;
                }
            }
        }

        self.recalculate(kern).await
    }

    pub async fn recalculate(&mut self, kern: &mut IncrDocClient) -> ZResult<()> {
        let elem = self.elem.clone().unwrap();

        match self.track_mode {
            TrackMode::Document => {
                self.retrack_pages(kern, elem).await?;
            } // TrackMode::Pages => todo!(),
        }

        let mut t = SvgBackendTask::default();
        let mut ctx = DomContext {
            tmpl: self.tmpl().clone(),
            stub: self.stub().clone(),
            module: kern.module(),
            t: &mut t,
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
        // render next doc_view
        // for pages that is not in the view, we use empty_page
        // otherwise, we keep document layout
        let next_doc_view = if let Some(t) = &kern.layout {
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            pages.clone()
        } else {
            vec![]
        };

        // for i in self.doc_view2.len()..next_doc_view.len() {
        //     let page = &next_doc_view[i];
        //     let elem = elem.clone();
        //     let mut dom_page = DomPage {
        //         elem,
        //         viewport: None,
        //     };
        //     dom_page.viewport = self.viewport;
        //     self.doc_view2.push(dom_page);
        // }
        if self.doc_view.len() > next_doc_view.len() {
            self.doc_view.truncate(next_doc_view.len());
        }
        for i in self.doc_view.len()..next_doc_view.len() {
            self.doc_view.push(DomPage::new_at(elem.hooked.clone(), i));
        }
        for (page, data) in self.doc_view.iter_mut().zip(next_doc_view.into_iter()) {
            page.track_data(data);
        }

        // render the glyphs
        let mut t = SvgBackendTask::default();
        let mut svg = Vec::<SvgText>::new();
        // var t = document.createElement('template');
        // t.innerHTML = html;
        // return t.content;
        svg.push(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml"><defs class="glyph">"#.into());
        let glyphs = kern.glyphs.iter();
        // skip the glyphs that are already rendered
        let new_glyphs = glyphs.skip(self.glyph_window);
        let glyph_defs = t.render_glyphs(new_glyphs.map(|(x, y)| (*x, y)));

        svg.extend(glyph_defs);
        svg.push("</defs></svg>".into());

        let svg = self.create_element(&SvgText::join(svg));
        let content = svg.first_element_child().unwrap();
        elem.resource_header.append_child(&content).unwrap();
        self.glyph_window = kern.glyphs.len();

        Ok(())
    }
}

pub struct DomContext<'m, 'a> {
    tmpl: HtmlTemplateElement,
    stub: Element,
    t: &'a mut SvgBackendTask,
    module: &'m Module,
}

impl<'m, 'a> DomContext<'m, 'a> {
    pub fn create_element(&self, html: &str) -> Element {
        let tmpl = &self.tmpl;
        tmpl.set_inner_html(html);
        tmpl.content().first_element_child().unwrap()
    }

    fn get_item(&self, id: &Fingerprint) -> Option<&'m VecItem> {
        self.module.get_item(id)
    }

    fn get_font(&self, id: &FontRef) -> Option<&'m FontItem> {
        self.module.get_font(id)
    }
}

struct DomPage {
    /// Index
    idx: usize,
    /// The element to track.
    elem: HtmlElement,
    /// The element to track.
    svg: Option<SvgsvgElement>,
    /// The page data
    data: Option<Page>,
    /// The page data
    next_data: Option<Page>,
    /// The viewport.
    viewport: Option<tiny_skia::Rect>,
    attached: Option<TypstPageElem>,
}

impl Drop for DomPage {
    fn drop(&mut self) {
        self.elem.remove();
    }
}

impl DomPage {
    pub fn new_at(elem: HtmlElement, idx: usize) -> Self {
        let me = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        elem.append_child(&me).unwrap();
        me.set_class_name("typst-dom-page");
        me.set_attribute("data-index", &idx.to_string()).unwrap();
        Self {
            idx,
            elem: me.dyn_into().unwrap(),
            svg: None,
            viewport: None,
            data: None,
            next_data: None,
            attached: None,
        }
    }

    fn track_data(&mut self, data: Page) {
        self.next_data = Some(data);
    }

    fn attach_dom(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
        elements: Element,
        data: Page,
    ) -> ZResult<()> {
        web_sys::console::log_2(
            &format!("attach {idx} {a:?}", idx = self.idx, a = self.attached).into(),
            &elements,
        );
        self.attached = Some(TypstPageElem::from_elem(ctx, elements, data));

        Ok(())
    }

    fn instantiate(&mut self, ctx: &mut DomContext<'_, '_>, data: &Page) -> ZResult<SvgsvgElement> {
        let mut svg = Vec::<SvgText>::new();

        svg.push(SvgText::Plain( format!(
            r#"<svg class="typst-svg-page" viewBox="0 0 {:.3} {:.3}" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">"#,
            data.size.x.0, data.size.y.0,
        )));

        let state = RenderState::new_size(data.size);
        let mut render_task = ctx.t.get_render_context(ctx.module);
        // render the document
        let mut svg_body = vec![];
        svg.push(SvgText::Content(
            render_task.render_item(state, &data.content),
        ));

        // attach the clip paths, and style defs

        svg.push(r#"<defs class="clip-path">"#.into());
        let patterns = ctx.t.render_patterns(ctx.module);

        let gradients = std::mem::take(&mut ctx.t.gradients);
        let gradients = gradients
            .values()
            .filter_map(|(_, id, _)| match ctx.module.get_item(id) {
                Some(VecItem::Gradient(g)) => Some((id, g.as_ref())),
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid gradient reference: {}", id.as_svg_id("g"));
                    #[allow(unreachable_code)]
                    None
                }
            });
        SvgBackend::gradients(gradients, &mut svg);
        SvgBackend::patterns(patterns.into_iter(), &mut svg);
        svg.push("</defs>".into());

        SvgBackend::style_defs(std::mem::take(&mut ctx.t.style_defs), &mut svg);

        // body
        svg.append(&mut svg_body);

        svg.push("</svg>".into());

        let svg = ctx.create_element(&SvgText::join(svg));
        Ok(svg.dyn_into().unwrap())
    }

    async fn do_recalculate(
        &mut self,
        ctx: &mut DomContext<'_, '_>,
        data: Page,
        viewport: Option<tiny_skia::Rect>,
    ) -> ZResult<()> {
        // calculate the width and height of the svg
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

        // update the state
        // self.doc_view = Some(next_doc_view);
        // self.glyph_window = kern.glyphs.len();

        // return the svg
        if self.elem.child_element_count() == 0 {
            let svg = self.instantiate(ctx, &data)?;
            self.elem.append_child(&svg).unwrap();
            self.attach_dom(ctx, svg.clone().into(), data)?;
            self.svg = Some(svg);
        } else {
            self.reflow(ctx, data).await?;
        }

        let svg = self.svg.clone().unwrap();
        let ctm = svg.get_ctm().unwrap();
        // self.elem.client_left()
        // self.elem.client_top()
        // let ts = tiny_skia::Transform::from_row(
        //     ctm.a(), ctm.b(), ctm.c(), ctm.d(), ctm.e(), ctm.f(),
        // );
        // ts regards clientLeft and clientTop relative to the window
        let cr = self.elem.get_bounding_client_rect();
        let ts = tiny_skia::Transform::from_row(
            ctm.a(),
            ctm.b(),
            ctm.c(),
            ctm.d(),
            ctm.e() - cr.left() as f32,
            ctm.f() - cr.top() as f32,
        );
        // web_sys::console::log_3(&format!("ctop {ctop:?}", ctop =
        // self.elem.client_top()).into(), &cr, &self.elem);
        self.viewport = viewport.and_then(|v| v.transform(ts));

        // const resolveCoord = (elem: SVGGElement, x: number, y: number) => {
        //     var matrix = elem.getScreenCTM();

        //     if (!matrix) {
        //       return { x: 0, y: 0 };
        //     }

        //     return {
        //       x: matrix.a * x + matrix.c * y + matrix.e - coordLeft,
        //       y: matrix.b * x + matrix.d * y + matrix.f - coordTop,
        //     };
        //   };
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
        web_sys::console::log_2(
            &format!("apply_diff {idx}", idx = self.idx).into(),
            &self.elem,
        );

        Ok(())
    }

    async fn repaint(&mut self, ctx: &mut DomContext<'_, '_>) -> ZResult<()> {
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
            let ts = tiny_skia::Transform::identity();
            let viewport = self.viewport.unwrap_or_else(|| {
                tiny_skia::Rect::from_xywh(0., 0., attached.size.x.0, attached.size.y.0).unwrap()
            });
            attached.repaint(ctx, ts, viewport);
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
struct TypstPageElem {
    show_visible: bool,
    stub: Element,
    elem_ctx: Element,
    g: TypstElem,
    size: Size,
    bbox: tiny_skia::Rect,
    clip_paths: Element,
    style_defs: Element,
}

impl TypstPageElem {
    fn from_elem(ctx: &mut DomContext<'_, '_>, elem_ctx: Element, data: Page) -> Self {
        let g: SvgGraphicsElement = elem_ctx.first_element_child().unwrap().dyn_into().unwrap();
        let stub: Element = ctx.stub.clone_node().unwrap().dyn_into().unwrap();
        let clip_paths = g.next_element_sibling().unwrap();
        let style_defs = clip_paths.next_element_sibling().unwrap();
        let attached = Self::attach(ctx, g.clone(), data.content);

        // do very lazy repaint/visibiliity on page elements
        elem_ctx.replace_with_with_node_1(&stub).unwrap();
        Self {
            show_visible: false,
            elem_ctx,
            g: attached,
            stub,
            size: data.size,
            bbox: tiny_skia::Rect::from_xywh(0., 0., data.size.x.0, data.size.y.0).unwrap(),
            clip_paths,
            style_defs,
        }
    }

    fn retrieve_bbox(&mut self) {
        self.g.retrieve_bbox_from_browser();
    }

    fn attach(ctx: &mut DomContext<'_, '_>, g: SvgGraphicsElement, data: Fingerprint) -> TypstElem {
        let item = ctx.get_item(&data).unwrap();

        // web_sys::console::log_2(&g, &format!("attach {a:?}", a =
        // data.as_svg_id("")).into());

        // let children = vec![];
        // GroupDom {
        //     g,
        //     f,
        //     children,
        // }
        let stub: Element = ctx.stub.clone_node().unwrap().dyn_into().unwrap();
        // g.replace_with_with_node_1(&stub).unwrap();

        let extra = match item {
            VecItem::Group(gr, size) => {
                let mut ch = g.first_element_child();

                let mut children = vec![];
                for (p, fg) in gr.0.iter() {
                    let Some(should_ch) = ch else {
                        web_sys::console::log_2(&g, &"Invalid group reference".into());
                        // panic!("Invalid group reference: {}", fg.as_svg_id("g"));
                        continue;
                    };

                    // skip translate g
                    let child = Self::attach(
                        ctx,
                        should_ch
                            .first_element_child()
                            .ok_or_else(|| {
                                web_sys::console::log_2(
                                    &should_ch,
                                    &format!("Invalid group translate: {:?}", item).into(),
                                );
                                panic!("Invalid group translate: {}", fg.as_svg_id("g"));
                            })
                            .unwrap()
                            .dyn_into()
                            .unwrap(),
                        *fg,
                    );
                    children.push((*p, child));

                    ch = should_ch.next_element_sibling();
                }
                TypstDomExtra::Group(GroupElem {
                    children,
                    size: *size,
                })
            }
            VecItem::Item(TransformedRef(trans, fg)) => {
                let child = Self::attach(ctx, g.clone(), *fg);

                TypstDomExtra::Item(TransformElem {
                    trans: trans.clone(),
                    child: Box::new(child),
                })
            }
            VecItem::Image(img) => TypstDomExtra::Image(ImageElem { size: img.size }),
            VecItem::Text(text) => TypstDomExtra::Text(TextElem {
                meta: text.clone(),
                upem: ctx.get_font(&text.shape.font).unwrap().units_per_em,
            }),
            VecItem::ContentHint(_) => TypstDomExtra::ContentHint(ContentHintElem { hint: ' ' }),
            VecItem::Link(_) => TypstDomExtra::Link(LinkElem {}),
            VecItem::Path(_) => TypstDomExtra::Path(PathElem {}),
            VecItem::None | VecItem::Color32(_) | VecItem::Gradient(_) | VecItem::Pattern(_) => {
                todo!()
            }
        };

        let mut ret = TypstElem {
            is_visible: true,
            browser_bbox_unchecked: true,
            stub,
            g,
            f: data,
            estimated_bbox: None,
            bbox: None,
            extra,
        };

        match &ret.extra {
            TypstDomExtra::ContentHint(_) => {
                ret.browser_bbox_unchecked = false;
            }
            TypstDomExtra::Text(g) => {
                ret.browser_bbox_unchecked = false;
                let shape = &g.meta.shape;
                let descender = ctx.get_font(&shape.font).unwrap().descender.0 * shape.size.0;
                let bbox = tiny_skia::Rect::from_xywh(
                    0.,
                    -shape.size.0 - descender,
                    g.meta.width().0,
                    shape.size.0,
                );
                ret.estimated_bbox = bbox;
            }
            _ => {}
        }
        ret
    }

    fn repaint(
        &mut self,
        _ctx: &mut DomContext<'_, '_>,
        ts: tiny_skia::Transform,
        viewport: tiny_skia::Rect,
    ) {
        let estimated_bbox = self.bbox.transform(ts);

        let should_visible = estimated_bbox
            .map(|estimated_bbox| estimated_bbox.intersect(&viewport).is_some())
            .unwrap_or(true);

        web_sys::console::log_1(
            &format!(
            "reflow_root({should_visible}) ts:{ts:?} estimated:{estimated_bbox:?} view:{viewport:?}"
        )
            .into(),
        );

        if should_visible != self.show_visible {
            self.show_visible = should_visible;
            if should_visible {
                self.stub.replace_with_with_node_1(&self.elem_ctx).unwrap();
            } else {
                self.elem_ctx.replace_with_with_node_1(&self.stub).unwrap();
                return;
            }
        }

        self.g.repaint(ts, viewport);
    }
}

static FETCH_BBOX_TIMES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static BBOX_SANITIZER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug)]
enum TypstDomExtra {
    Group(GroupElem),
    Item(TransformElem),
    Image(ImageElem),
    Text(TextElem),
    Path(PathElem),
    Link(LinkElem),
    ContentHint(ContentHintElem),
}

#[derive(Debug)]
struct TypstElem {
    is_visible: bool,
    browser_bbox_unchecked: bool,
    stub: Element,
    g: SvgGraphicsElement,
    f: Fingerprint,
    estimated_bbox: Option<tiny_skia::Rect>,
    bbox: Option<tiny_skia::Rect>,
    extra: TypstDomExtra,
}

impl TypstElem {
    fn repaint(&mut self, ts: tiny_skia::Transform, viewport: tiny_skia::Rect) {
        use TypstDomExtra::*;
        if matches!(self.extra, ContentHint(_)) {
            return; // always visible
        }

        if self.browser_bbox_unchecked {
            let cnt_check = BBOX_SANITIZER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let fetch_for_sanitizing = (cnt_check & 511) == 0;

            if self.estimated_bbox.is_none() || fetch_for_sanitizing {
                self.retrieve_bbox_from_browser();
                if self.browser_bbox_unchecked {
                    return;
                }

                if fetch_for_sanitizing && self.estimated_bbox.is_some() {
                    self.ensure_bbox_is_well_estimated();
                }
            }
        }

        let new_rect = self.bbox.or(self.estimated_bbox).unwrap();
        let should_visible = new_rect
            .transform(ts)
            .map(|new_rect| new_rect.intersect(&viewport).is_some())
            .unwrap_or(true);

        if should_visible != self.is_visible {
            let (x, y) = (&self.stub, &self.g);
            if should_visible {
                x.replace_with_with_node_1(y).unwrap();
            } else {
                y.replace_with_with_node_1(x).unwrap();
            };

            self.is_visible = should_visible;
            if !should_visible {
                return;
            }
        }

        match &mut self.extra {
            Group(g) => {
                for (p, child) in g.children.iter_mut() {
                    let ts = ts.pre_translate(p.x.0, p.y.0);
                    child.repaint(ts, viewport);
                }
            }
            Item(g) => {
                let trans = g.trans.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                // todo: intersect viewport
                // if let TransformItem::Clip(c) = g.trans {

                // }
                g.child.repaint(ts, viewport);
            }
            _ => {}
        }
    }

    fn retrieve_bbox_from_browser(&mut self) {
        if !self.browser_bbox_unchecked {
            return;
        }

        let bbox = self.g.get_b_box().unwrap();
        FETCH_BBOX_TIMES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if bbox.width() == 0. && bbox.height() == 0. {
            return; // wait for next browser tick to retrieve
        }
        let mut ccbbox =
            tiny_skia::Rect::from_xywh(bbox.x(), bbox.y(), bbox.width(), bbox.height());
        if let TypstDomExtra::Text(t) = &self.extra {
            let ppem = t.meta.shape.ppem(t.upem.0).0;
            ccbbox =
                ccbbox.and_then(|r| r.transform(tiny_skia::Transform::from_scale(ppem, -ppem)));
        }
        // web_sys::console::log_2(&bbox, &format!("retrieved_bbox {a:?} {ccbbox:?}", a
        // = self.f.as_svg_id("")).into());
        self.bbox = ccbbox;
        self.browser_bbox_unchecked = false;

        if let TypstDomExtra::Group(g) = &mut self.extra {
            for (_, child) in g.children.iter_mut() {
                child.retrieve_bbox_from_browser();
            }
        }
    }

    fn ensure_bbox_is_well_estimated(&self) {
        static WARN_ONCE: AtomicBool = AtomicBool::new(false);
        let bbox = self.bbox.unwrap().round();
        let estmiated = self.estimated_bbox.unwrap().round_out();
        if estmiated
            .zip(bbox)
            .map(|(a, b)| a.contains(&b))
            .unwrap_or(false)
        {
            return;
        }

        if !WARN_ONCE.swap(true, std::sync::atomic::Ordering::SeqCst) {
            web_sys::console::warn_2(
                &format!(
                    "bbox may not be well estimated: estimated_bbox:{:?} bbox:{:?}, item: {:?}, kind: {:?}, elem:",
                    estmiated, bbox,
                    self.f.as_svg_id(""),
                    self.extra,
                )
                .into(),
                &self.g,
            );
        }
    }
}

#[derive(Debug)]
struct ImageElem {
    size: Size,
}
#[derive(Debug)]
struct TextElem {
    upem: Scalar,
    meta: TextItem,
}
#[derive(Debug)]
struct PathElem {}
#[derive(Debug)]
struct LinkElem {}
#[derive(Debug)]
struct ContentHintElem {
    hint: char,
}

#[derive(Debug)]
struct TransformElem {
    trans: TransformItem,
    child: Box<TypstElem>,
}

#[derive(Debug)]
struct GroupElem {
    children: Vec<(Point, TypstElem)>,
    size: Option<Size>,
}
