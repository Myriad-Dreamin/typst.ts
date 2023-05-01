#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::renderer::session::RenderSessionOptions;

    fn render_test_template(point: &str, artifact: &[u8], format: &str) {
        let artifact = artifact.into();

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let mut renderer = crate::tests::get_renderer();

        let mut session = renderer.create_session(artifact, Some(RenderSessionOptions{
            pixel_per_pt: Some(1.0),
            background_color: Some("343541".to_string()),
            format: Some(format.to_string()),
        })).unwrap();

        let canvas = window
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let sizes = session.doc.pages[0].size();
        canvas.set_width(sizes.x.to_pt() as u32);
        canvas.set_height(sizes.y.to_pt() as u32);

        let context = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();

        let serde_task = {
            let start = performance.now();
            let res = renderer
                .render_page_to_canvas(&session, &context, None)
                .unwrap();
            web_sys::console::log_2(&"textContent".into(), &res);
            let end = performance.now();

            (end - start, artifact)
        };

        window
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&canvas)
            .unwrap();

        console_log!("canvas {point} {}ms", serde_task.0);
    }

    #[wasm_bindgen_test]
    fn render_ir_test() {
        render_test_template(
            "main_artifact",
            include_bytes!("../../main.artifact_ir.bin").as_slice(),
            "ir",
        );
    }

    #[wasm_bindgen_test]
    fn render_json_test() {
        render_test_template(
            "main_artifact",
            include_bytes!("../../main.artifact.json").as_slice(),
            "js",
        );
    }

    // #[wasm_bindgen_test]
    // fn render_cv_test() {
    //     render_test_template(
    //         "cv_artifact",
    //         include_bytes!("../../cv.artifact.json").as_slice(),
    //     );
    // }
}
