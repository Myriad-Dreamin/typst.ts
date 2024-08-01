use crate::RenderSession;
use crate::TypstRenderer;
use js_sys::Uint8Array;
use reflexo_typst2vec::geom::{Axes, Scalar};
use typst_ts_core::error::prelude::*;
use typst_ts_core::svg::SvgDataSelection;
use typst_ts_core::svg::{DefaultExportFeature, SvgExporter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl RenderSession {
    pub fn render_in_window(
        &mut self,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        use reflexo_typst2vec::geom::Rect;

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
    pub fn render_svg_diff(
        &mut self,
        session: &mut RenderSession,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        session.render_in_window(rect_lo_x, rect_lo_y, rect_hi_x, rect_hi_y)
    }

    pub fn svg_data(&mut self, session: &mut RenderSession, parts: Option<u32>) -> ZResult<String> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;

        let client = session.client.lock().unwrap();
        let Some(layout) = &client.layout else {
            return Err(error_once!("Renderer.MissingLayout"));
        };

        let view = layout.pages(client.module()).unwrap();

        let parts = parts.map(|parts| SvgDataSelection {
            body: 0 != (parts & (1 << 0)),
            defs: 0 != (parts & (1 << 1)),
            css: 0 != (parts & (1 << 2)),
            js: 0 != (parts & (1 << 3)),
        });

        let svg = UsingExporter::render_flat_svg(view.module(), view.pages(), parts);

        Ok(svg)
    }

    pub fn get_customs(&self, session: &RenderSession) -> Option<js_sys::Array> {
        let client = session.client.lock().unwrap();
        let layout = client.layout.clone();

        layout.map(|layout| {
            let view = layout.pages(client.module()).unwrap();
            view.customs()
                .map(|(k, v)| {
                    [JsValue::from_str(k), Uint8Array::from(v.as_ref()).into()]
                        .into_iter()
                        .collect::<js_sys::Array>()
                })
                .collect::<js_sys::Array>()
        })
    }

    pub fn render_svg(&self, session: &RenderSession, root: web_sys::HtmlElement) -> ZResult<bool> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        // todo: leaking abstraction
        let mut client = session.client.lock().unwrap();
        let layouts = client.doc.layouts[0].by_scalar().unwrap();
        let mut layout = layouts.first().unwrap();

        // base scale = 2
        let base_cw = root.client_width() as f32;

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

        let layout = layout.clone();
        client.set_layout(layout.1.clone());

        // caching updation
        let applying = format!("{}px", layout.0 .0);
        let applied = root.get_attribute("data-applied-width");
        if applied.is_some() && applied.unwrap() == applying {
            // console_log!("already applied {}", applying);
            return Ok(true);
        }

        let view = layout.1.pages(client.module()).unwrap();
        let svg = UsingExporter::render_flat_svg(view.module(), view.pages(), None);
        drop(client);

        // console_log!("base_cw {}", base_cw);

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

        Ok(false)
    }
}
