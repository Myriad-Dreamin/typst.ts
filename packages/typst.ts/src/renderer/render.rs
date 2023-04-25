#[cfg(test)]
// #[cfg(target_arch = "wasm32")]
mod tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn render_test() {
        let artifact = include_bytes!("../../main.artifact.json");
        let artifact = String::from_utf8_lossy(artifact);

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let mut renderer = crate::tests::get_renderer();

        let mut session = renderer.create_session(artifact.to_string(), None).unwrap();
        session.background_color = "343541".to_string();
        session.pixel_per_pt = 1.0;

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
            renderer
                .render_page_to_canvas(&session, &context, None)
                .unwrap();
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

        console_log!("canvas {}ms", serde_task.0);
    }
}
