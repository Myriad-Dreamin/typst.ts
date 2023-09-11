use crate::RenderSession;
use crate::TypstRenderer;
use typst_ts_core::error::prelude::*;
use typst_ts_core::vector::geom::Axes;
use typst_ts_core::vector::geom::Scalar;
use typst_ts_svg_exporter::flat_ir::LayoutRegionNode;
use typst_ts_svg_exporter::{DefaultExportFeature, SvgExporter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl RenderSession {
    pub fn reset(&mut self) {
        self.client = Default::default();
    }

    pub fn render_svg_in_window(
        &mut self,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        use typst_ts_core::vector::geom::Rect;

        let mut client = self.client.lock().unwrap();
        let mut svg_kern = self.svg_kern.lock().unwrap();

        svg_kern.render_in_window(
            &mut client,
            Rect {
                lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
                hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
            },
        )
    }
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn render_svg(&self, session: &RenderSession, root: web_sys::HtmlElement) -> ZResult<()> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        // todo: leaking abstraction
        let client = session.client.lock().unwrap();
        let layouts = client.doc.layouts[0].by_scalar().unwrap();
        let layout = layouts.first().unwrap();

        // base scale = 2
        let base_cw = root.client_width() as f32;

        let render = |layout: &(Scalar, LayoutRegionNode)| {
            let applying = format!("{}px", layout.0 .0);

            let applied = root.get_attribute("data-applied-width");
            if applied.is_some() && applied.unwrap() == applying {
                // console_log!("already applied {}", applying);
                return Ok(());
            }

            let view = layout.1.pages(client.module()).unwrap();

            let svg = UsingExporter::render_flat_svg(view.module(), view.pages());
            root.set_inner_html(&svg);
            let window = web_sys::window().unwrap();
            if let Ok(proc) = js_sys::Reflect::get(&window, &JsValue::from_str("typstProcessSvg")) {
                proc.dyn_ref::<js_sys::Function>()
                    .unwrap()
                    .call1(&JsValue::NULL, &root.first_element_child().unwrap())
                    .unwrap();
            }

            root.set_attribute("data-applied-width", &applying).unwrap();
            // console_log!("applied {}", applying);

            Ok(())
        };

        // console_log!("base_cw {}", base_cw);

        const EPS: f32 = 1e-2;

        if layout.0 .0 < base_cw + EPS {
            return render(layout);
        }

        let layout = layouts.last().unwrap();

        if layout.0 .0 + EPS > base_cw {
            return render(layout);
        }

        for layout in layouts {
            if layout.0 .0 < base_cw + EPS {
                return render(layout);
            }
        }

        Ok(())
    }
}
