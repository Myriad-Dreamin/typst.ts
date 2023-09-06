use crate::TypstRenderer;
use typst_ts_core::error::prelude::*;
use typst_ts_core::vector::geom::Axes;
use typst_ts_core::vector::geom::Scalar;
use typst_ts_svg_exporter::flat_ir::LayoutRegionNode;
use typst_ts_svg_exporter::IncrSvgDocClient;
use typst_ts_svg_exporter::{DefaultExportFeature, SvgExporter};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SvgSession {
    client: IncrSvgDocClient,
}

#[wasm_bindgen]
impl SvgSession {
    pub fn reset(&mut self) {
        self.client = Default::default();
    }

    #[wasm_bindgen(getter)]
    pub fn doc_width(&self) -> f32 {
        self.client.kern().doc_width().unwrap_or_default()
    }

    #[wasm_bindgen(getter)]
    pub fn doc_height(&self) -> f32 {
        self.client.kern().doc_height().unwrap_or_default()
    }

    pub fn source_span(&self, path: &[u32]) -> ZResult<Option<String>> {
        self.client.kern().source_span(path)
    }

    pub fn merge_delta(&mut self, delta: &[u8]) -> ZResult<()> {
        use typst_ts_core::vector::stream::BytesModuleStream;

        let delta = BytesModuleStream::from_slice(delta).checkout_owned();

        #[cfg(feature = "debug_delta_update")]
        crate::utils::console_log!(
            "module counts: {:?},{:?},{:?}",
            delta.glyphs.len(),
            delta.item_pack.0.len(),
            delta.layouts.len()
        );

        self.client.merge_delta(delta);
        Ok(())
    }

    pub fn render_in_window(
        &mut self,
        rect_lo_x: f32,
        rect_lo_y: f32,
        rect_hi_x: f32,
        rect_hi_y: f32,
    ) -> String {
        use typst_ts_core::vector::geom::Rect;

        self.client.render_in_window(Rect {
            lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
            hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
        })
    }
}

#[wasm_bindgen]
impl TypstRenderer {
    pub fn create_svg_session(&self, artifact_content: &[u8]) -> ZResult<SvgSession> {
        use typst_ts_svg_exporter::MultiSvgDocument;

        let doc = MultiSvgDocument::from_slice(artifact_content);
        Ok(SvgSession {
            client: IncrSvgDocClient::new(doc),
        })
    }

    pub fn create_empty_svg_session(&self) -> ZResult<SvgSession> {
        Ok(SvgSession {
            client: Default::default(),
        })
    }

    pub fn render_svg(&self, session: &SvgSession, root: web_sys::HtmlDivElement) -> ZResult<()> {
        type UsingExporter = SvgExporter<DefaultExportFeature>;
        // todo: leaking abstraction
        let layouts = session.client.kern.doc.layouts.by_scalar().unwrap();
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

            let view = layout.1.pages(&session.client.kern.doc.module).unwrap();

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

        // console_log!(
        //     "layouts {:?}",
        //     session
        //         .client
        //         .doc
        //         .layouts
        //         .iter()
        //         .map(|x| x.0)
        //         .collect::<Vec<_>>()
        // );

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
