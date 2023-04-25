#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn artifact_deserialization() {
        let artifact = include_bytes!("../../main.artifact.json");
        let artifact = String::from_utf8_lossy(artifact);

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let mut renderer = crate::tests::get_renderer();

        let session = renderer.create_session(artifact.to_string(), None).unwrap();

        let canvas = window
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

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

        console_log!("canvas {}ms", serde_task.0);
    }
}
