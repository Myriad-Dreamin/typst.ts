#![allow(dead_code)]

use std::sync::atomic::AtomicBool;

use typst_ts_core::{
    hash::Fingerprint,
    vector::{
        incr::IncrDocClient,
        vm::{RenderState, RenderVm},
    },
};
use typst_ts_svg_exporter::{
    ir::{self, Page, TransformedRef, VecItem},
    Module, SvgExporter, SvgTask, SvgText,
};
use web_sys::{wasm_bindgen::JsCast, Element, SvgGraphicsElement};

use crate::{dom::*, factory::XmlFactory, DomContext, HookedElement};

/// The feature set which is used for exporting incremental rendered svg.
pub struct IncrementalSvgExportFeature;

impl typst_ts_svg_exporter::ExportFeature for IncrementalSvgExportFeature {
    const ENABLE_INLINED_SVG: bool = false;
    const ENABLE_TRACING: bool = false;
    const SHOULD_ATTACH_DEBUG_INFO: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = false;
    const USE_STABLE_GLYPH_ID: bool = true;
    const SHOULD_RASTERIZE_TEXT: bool = false;
    const WITH_BUILTIN_CSS: bool = false;
    const WITH_RESPONSIVE_JS: bool = false;
    const AWARE_HTML_ENTITY: bool = true;
}

type Exporter = SvgExporter<IncrementalSvgExportFeature>;
pub type Vec2SvgPass = SvgTask<'static, IncrementalSvgExportFeature>;

#[derive(Default)]
pub struct SvgBackend {
    vec2svg: Vec2SvgPass,

    /// Glyphs that has already committed to the DOM.
    /// Assmuing glyph_window = N, then `self.doc.module.glyphs[..N]` are
    /// committed.
    pub glyph_window: usize,

    factory: XmlFactory,
}

impl SvgBackend {
    pub fn reset(&mut self) {
        self.glyph_window = 0;
    }

    fn create_element(&self, html: &str) -> Element {
        self.factory.create_element(html)
    }

    pub(crate) fn populate_glyphs(&mut self, ctx: &mut IncrDocClient, elem: &HookedElement) {
        let mut svg = Vec::<SvgText>::new();
        // var t = document.createElement('template');
        // t.innerHTML = html;
        // return t.content;
        svg.push(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml"><defs class="glyph">"#.into());
        let glyphs = ctx.glyphs.iter();
        // skip the glyphs that are already rendered
        let new_glyphs = glyphs.skip(self.glyph_window);
        let glyph_defs = self.vec2svg.render_glyphs(new_glyphs.map(|(x, y)| (*x, y)));

        svg.extend(glyph_defs);
        svg.push("</defs></svg>".into());

        let svg = self.create_element(&SvgText::join(svg));
        let content = svg.first_element_child().unwrap();

        elem.resource_header.append_child(&content).unwrap();
        self.glyph_window = ctx.glyphs.len();
    }

    pub(crate) fn render_page(
        &mut self,
        module: &Module,
        page: &Page,
        parent: &Element,
    ) -> Element {
        let mut render_task = self.vec2svg.get_render_context(module);

        let mut g = vec!["<g>".into()];

        // render the document
        let state = RenderState::new_size(page.size);
        g.push(SvgText::Content(
            render_task.render_item(state, &page.content),
        ));

        // attach the clip paths, and style defs

        g.push(r#"<defs class="clip-path">"#.into());
        let patterns = self.vec2svg.render_patterns(module);

        let gradients = std::mem::take(&mut self.vec2svg.gradients);
        let gradients = gradients
            .values()
            .filter_map(|(_, id, _)| match module.get_item(id) {
                Some(VecItem::Gradient(g)) => Some((id, g.as_ref())),
                _ => {
                    // #[cfg(debug_assertions)]
                    panic!("Invalid gradient reference: {}", id.as_svg_id("g"));
                    #[allow(unreachable_code)]
                    None
                }
            });
        // todo: remove Exporter usages
        Exporter::gradients(gradients, &mut g);
        Exporter::patterns(patterns.into_iter(), &mut g);
        g.push("</defs>".into());

        Exporter::style_defs(std::mem::take(&mut self.vec2svg.style_defs), &mut g);

        g.push(r#"</g>"#.into());

        //Note: create in svg namespace
        parent.set_inner_html(&SvgText::join(g));
        parent.first_element_child().unwrap()
    }
}

impl TypstPageElem {
    pub fn attach_svg(
        ctx: &mut DomContext<'_, '_>,
        g: SvgGraphicsElement,
        data: Fingerprint,
    ) -> TypstElem {
        let item = ctx.get_item(&data).unwrap();

        // web_sys::console::log_2(&g, &format!("attach {a:?}", a =
        // data.as_svg_id("")).into());

        // let children = vec![];
        // GroupDom {
        //     g,
        //     f,
        //     children,
        // }
        let stub = ctx.create_stub();
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
                    let child = Self::attach_svg(
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
                let ch = g.last_element_child();

                let child = Self::attach_svg(
                    ctx,
                    ch.ok_or_else(|| {
                        web_sys::console::log_2(
                            &g,
                            &format!("Invalid item reference: {:?}", item).into(),
                        );
                        panic!("Invalid item reference: {}", fg.as_svg_id("g"));
                    })
                    .unwrap()
                    .dyn_into()
                    .unwrap(),
                    *fg,
                );

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
            is_svg_visible: true,
            browser_bbox_unchecked: true,
            stub,
            g,
            f: data,
            estimated_bbox: None,
            bbox: None,
            extra,
            canvas: None,
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

    pub fn repaint_svg(
        &mut self,
        _ctx: &mut DomContext<'_, '_>,
        ts: tiny_skia::Transform,
        viewport: tiny_skia::Rect,
    ) {
        self.g.repaint_svg(ts, viewport);
    }
}

pub static FETCH_BBOX_TIMES: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
static BBOX_SANITIZER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl TypstElem {
    fn repaint_svg(&mut self, ts: tiny_skia::Transform, viewport: tiny_skia::Rect) {
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

        let bbox = self.bbox.as_deref().cloned();
        let bbox = bbox.or(self.estimated_bbox).unwrap();
        let should_visible = bbox
            .transform(ts)
            .map(|new_rect| new_rect.intersect(&viewport).is_some())
            .unwrap_or(true);

        if should_visible != self.is_svg_visible {
            let (x, y) = (&self.stub, &self.g);
            if should_visible {
                x.replace_with_with_node_1(y).unwrap();
            } else {
                y.replace_with_with_node_1(x).unwrap();
            };

            self.is_svg_visible = should_visible;
        }

        if !should_visible {
            return;
        }

        match &mut self.extra {
            Group(g) => {
                for (p, child) in g.children.iter_mut() {
                    let ts = ts.pre_translate(p.x.0, p.y.0);
                    child.repaint_svg(ts, viewport);
                }
            }
            Item(g) => {
                let trans = g.trans.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                // todo: intersect viewport
                // if let TransformItem::Clip(c) = g.trans {

                // }
                g.child.repaint_svg(ts, viewport);
            }
            _ => {}
        }
    }

    pub fn retrieve_bbox_from_browser(&mut self) {
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
        self.bbox = ccbbox.map(Box::new);
        self.browser_bbox_unchecked = false;

        if let TypstDomExtra::Group(g) = &mut self.extra {
            for (_, child) in g.children.iter_mut() {
                child.retrieve_bbox_from_browser();
            }
        }
    }

    fn ensure_bbox_is_well_estimated(&self) {
        static WARN_ONCE: AtomicBool = AtomicBool::new(false);
        let bbox = self.bbox.as_ref().map(|b| b.round()).unwrap();
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
