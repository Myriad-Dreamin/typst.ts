#![allow(dead_code)]

use reflexo::hash::Fingerprint;
use reflexo::vector::ir::{self, Module, Page, TransformedRef, VecItem};
use reflexo::vector::{incr::IncrDocClient, vm::RenderVm};
use reflexo_vec2canvas::BBoxAt;
use reflexo_vec2svg::{SvgExporter, SvgTask, SvgText};
use web_sys::{wasm_bindgen::JsCast, Element, SvgGraphicsElement};

use crate::{dom::*, factory::XmlFactory, DomContext};

/// The feature set which is used for exporting incremental rendered svg.
pub struct IncrementalSvgExportFeature;

impl reflexo_vec2svg::ExportFeature for IncrementalSvgExportFeature {
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

    pub(crate) fn populate_glyphs(&mut self, ctx: &mut IncrDocClient) -> Option<String> {
        if ctx.glyphs.len() <= self.glyph_window {
            return None;
        }

        let mut svg = Vec::<SvgText>::new();

        svg.push(r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml"><defs class="glyph">"#.into());
        let glyphs = ctx.glyphs.iter();
        // skip the glyphs that are already rendered
        let new_glyphs = glyphs.skip(self.glyph_window);
        let glyph_defs = self.vec2svg.render_glyphs(new_glyphs.map(|(x, y)| (*x, y)));

        svg.extend(glyph_defs);
        svg.push("</defs></svg>".into());

        self.glyph_window = ctx.glyphs.len();

        Some(SvgText::join(svg))
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
        g.push(SvgText::Content(render_task.render_item(&page.content)));

        // attach the clip paths, and style defs

        g.push(r#"<defs class="clip-path">"#.into());
        let patterns = self.vec2svg.render_patterns(module);

        let gradients = std::mem::take(&mut self.vec2svg.gradients);
        let gradients = gradients.iter().filter_map(|id| match module.get_item(id) {
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

        let stub = ctx.create_stub();
        // g.replace_with_with_node_1(&stub).unwrap();

        let extra = match item {
            VecItem::Group(gr) => {
                let mut ch = g.first_element_child();

                let mut children = vec![];
                for (p, fg) in gr.0.iter() {
                    #[cfg(feature = "debug_attach")]
                    web_sys::console::log_3(
                        &format!(
                            "attach {a:?} -> {b:?} {c:?}",
                            a = data.as_svg_id("g"),
                            b = fg.as_svg_id("g"),
                            c = p
                        )
                        .into(),
                        ch.as_ref()
                            .map(|e| e.dyn_ref().unwrap())
                            .unwrap_or(&JsValue::UNDEFINED),
                        &g,
                    );

                    let Some(should_ch) = ch else {
                        web_sys::console::log_2(&"Invalid group reference".into(), &g);
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
                TypstDomExtra::Group(GroupElem { children })
            }
            VecItem::Item(TransformedRef(trans, fg)) => {
                let ch = g
                    .last_element_child()
                    .ok_or_else(|| {
                        web_sys::console::log_2(
                            &g,
                            &format!("Invalid item reference: {:?}", item).into(),
                        );
                        panic!("Invalid item reference: {}", fg.as_svg_id("g"));
                    })
                    .unwrap();

                #[cfg(feature = "debug_attach")]
                web_sys::console::log_3(
                    &format!(
                        "attach {a:?} -> {b:?} {c:?}",
                        a = data.as_svg_id("g"),
                        b = fg.as_svg_id("g"),
                        c = trans
                    )
                    .into(),
                    &ch,
                    &g,
                );

                let child = Self::attach_svg(
                    ctx,
                    ch.first_element_child()
                        .ok_or_else(|| {
                            web_sys::console::log_2(
                                &g,
                                &format!("Invalid item translate: {:?}", item).into(),
                            );
                            panic!("Invalid item translate: {}", fg.as_svg_id("g"));
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
            VecItem::Html(_) => TypstDomExtra::Html(HtmlElem {}),
            VecItem::None
            | VecItem::ColorTransform(_)
            | VecItem::Color32(_)
            | VecItem::Gradient(_)
            | VecItem::Pattern(_) => {
                todo!()
            }
        };

        TypstElem {
            is_svg_visible: true,
            is_canvas_painted: false,
            stub,
            g,
            f: data,
            extra,
            canvas: None,
        }
    }

    pub fn repaint_svg(
        &mut self,
        _ctx: &mut DomContext<'_, '_>,
        ts: tiny_skia::Transform,
        viewport: ir::Rect,
    ) {
        self.g.repaint_svg(ts, viewport);
    }
}

pub static FETCH_BBOX_TIMES: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
static BBOX_SANITIZER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

impl TypstElem {
    /// Repaint svg API will retrun a new viewport if it is updated.
    ///
    /// The idea is that: the element visible before will be overrided by the
    /// latter ones, so we should update the viewport to the union of all
    /// previous ones and repaint the latter elements accordingly.
    fn repaint_svg(
        &mut self,
        ts: tiny_skia::Transform,
        mut viewport: ir::Rect,
    ) -> Option<ir::Rect> {
        use TypstDomExtra::*;
        if matches!(self.extra, ContentHint(_)) {
            return None; // always visible
        }

        let bbox = self.canvas.as_ref().unwrap().bbox_at(ts);
        let should_visible = bbox
            .map(|new_rect| new_rect.intersect(&viewport).is_intersected())
            .unwrap_or(true);
        // web_sys::console::log_2(
        //     &"bbox".into(),
        //     &format!(
        //         "{:?} -> ({bbox:?} & {viewport:?}) = {should_visible}",
        //         self.f.as_svg_id("g")
        //     )
        //     .into(),
        // );

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
            return None;
        }

        match &mut self.extra {
            Group(g) => {
                for (p, child) in g.children.iter_mut() {
                    let ts = ts.pre_translate(p.x.0, p.y.0);
                    if let Some(updated) = child.repaint_svg(ts, viewport) {
                        viewport = updated;
                    }
                }

                Some(viewport)
            }
            Item(g) => {
                let trans = g.trans.clone();
                let trans: ir::Transform = trans.into();
                let ts = ts.pre_concat(trans.into());
                // todo: intersect viewport
                // if let TransformItem::Clip(c) = g.trans {

                // }
                g.child.repaint_svg(ts, viewport)
            }
            _ => bbox.map(|bbox| viewport.union(&bbox)),
        }
    }
}
