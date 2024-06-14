use reflexo_vec2canvas::{CanvasElem, CanvasNode, CanvasOp, CanvasTask, ExportFeature};
use typst_ts_core::{
    error::prelude::ZResult,
    vector::{
        ir::{Module, Page},
        vm::RenderVm,
    },
};

use crate::dom::*;

/// The default feature set which is used for exporting full-fledged svg.
pub struct DefaultExportFeature;

impl ExportFeature for DefaultExportFeature {
    const ENABLE_TRACING: bool = false;
    const SHOULD_RENDER_TEXT_ELEMENT: bool = false;
}

type Vec2Canvas = CanvasTask<DefaultExportFeature>;

#[derive(Default)]
pub struct CanvasBackend {
    vec2canvas: Vec2Canvas,

    pub pixel_per_pt: f32,
}

impl CanvasBackend {
    pub fn reset(&mut self) {
        self.pixel_per_pt = 3.;
    }

    pub fn render_page(&mut self, module: &Module, page: &Page) -> ZResult<CanvasNode> {
        // todo: incremental
        let mut ct = self.vec2canvas.fork_canvas_render_task(module);

        Ok(ct.render_item(&page.content))
    }
}

impl TypstPageElem {
    pub fn attach_canvas(&mut self, g: CanvasNode) {
        self.g.attach_canvas(g)
    }

    pub async fn repaint_canvas(
        &mut self,
        ts: tiny_skia::Transform,
        panel: &web_sys::CanvasRenderingContext2d,
    ) {
        self.g.repaint_canvas(ts, panel, true).await;
    }
}

impl TypstElem {
    pub fn attach_canvas(&mut self, g: CanvasNode) {
        use TypstDomExtra::*;

        self.canvas = Some(g.clone());

        match &mut self.extra {
            Group(gr) => {
                let CanvasElem::Group(c) = g.as_ref() else {
                    panic!("Invalid group canvas: {}", self.f.as_svg_id("g"));
                };

                let this = gr.children.iter_mut().map(|t| &mut t.1);
                let this_c = c.inner.iter().map(|t| t.1.clone());

                for (elem, c_elem) in this.zip(this_c) {
                    elem.attach_canvas(c_elem);
                }
            }
            Item(ch) => {
                let mut g = g.as_ref();
                if let CanvasElem::Clip(c) = g {
                    g = &c.inner;
                };
                let CanvasElem::Group(c) = g else {
                    panic!("Invalid item canvas: {}", self.f.as_svg_id("g"));
                };
                if c.inner.len() != 1 {
                    panic!("Invalid item canvas length: {}", self.f.as_svg_id("g"));
                }

                ch.child.attach_canvas(c.inner[0].1.clone());
            }
            Link(..) | Image(..) | Text(..) | Path(..) | ContentHint(..) => {}
        };
    }

    #[async_recursion::async_recursion(?Send)]
    async fn repaint_canvas(
        &mut self,
        ts: tiny_skia::Transform,
        panel: &web_sys::CanvasRenderingContext2d,
        visible: bool,
    ) {
        use TypstDomExtra::*;

        match &mut self.extra {
            ContentHint(_) => {}
            Group(i) => {
                let visible = visible && self.is_svg_visible;
                if !visible {
                    self.canvas.as_ref().unwrap().realize(ts, panel).await;
                    return;
                }

                let g = self.canvas.as_deref().unwrap();

                let CanvasElem::Group(c) = g else {
                    panic!("Invalid group canvas: {}", self.f.as_svg_id("g"));
                };

                let ts = ts.pre_concat(*c.ts.as_ref());

                for (p, child) in i.children.iter_mut() {
                    let ts = ts.pre_translate(p.x.0, p.y.0);
                    child.repaint_canvas(ts, panel, visible).await;
                }
            }
            Item(i) => {
                let visible = visible && self.is_svg_visible;
                if !visible {
                    self.canvas.as_ref().unwrap().realize(ts, panel).await;
                    return;
                }

                let g = self.canvas.as_deref().unwrap();

                let (g, _clip_guard) = if let CanvasElem::Clip(clip) = g {
                    (clip.inner.as_ref(), Some(clip.realize_with(ts, panel)))
                } else {
                    (g, None)
                };

                let CanvasElem::Group(c) = g else {
                    panic!("Invalid group canvas: {}", self.f.as_svg_id("g"));
                };

                let ts = ts.pre_concat(*c.ts.as_ref());

                // todo: intersect viewport
                // if let TransformItem::Clip(c) = g.trans {

                // }
                i.child.repaint_canvas(ts, panel, visible).await;
            }
            _ => {
                // web_sys::console::log_2(
                //     &format!(
                //         "repaint_canvas_partial: {} {} {}",
                //         self.f.as_svg_id("g"),
                //         visible,
                //         self.is_svg_visible
                //     )
                //     .into(),
                //     &self.g,
                // );

                if visible || self.is_svg_visible {
                    return;
                }
                self.canvas.as_ref().unwrap().realize(ts, panel).await
            }
        }
    }
}
