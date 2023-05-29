#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_test_common::{MAIN_ARTIFACT_IR, MAIN_ARTIFACT_JSON};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::session::RenderSessionOptions;

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    #[derive(Serialize, Deserialize)]
    struct CanvasRenderTestPointMeta {
        time_used: String,
        data_content_hash: String,
        text_content_hash: String,
    }

    #[derive(Serialize, Deserialize)]
    struct CanvasRenderTestPoint {
        kind: String,
        name: String,
        meta: CanvasRenderTestPointMeta,
        verbose: HashMap<String, String>,
    }

    fn render_test_template(point: &str, artifact: &[u8], format: &str) {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let canvas = window
            .document()
            .unwrap()
            .create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let serde_task = {
            let start = performance.now();

            let mut renderer = crate::tests::get_renderer();

            let session = renderer
                .create_session(
                    artifact,
                    Some(RenderSessionOptions {
                        pixel_per_pt: Some(3.0),
                        background_color: Some("ffffff".to_string()),
                        format: Some(format.to_string()),
                    }),
                )
                .unwrap();

            let sizes = session.doc.pages[0].size();
            canvas.set_width((sizes.x.to_pt() * 3.).ceil() as u32);
            canvas.set_height((sizes.y.to_pt() * 3.).ceil() as u32);

            let context: web_sys::CanvasRenderingContext2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            let res = renderer
                .render_page_to_canvas(&session, &context, None)
                .unwrap();
            let end = performance.now();

            let data_content = canvas.to_data_url_with_type("image/jpeg").unwrap();

            let text_content = js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();

            web_sys::console::log_3(
                &">>> typst_ts_test_capture".into(),
                &serde_json::to_string(&CanvasRenderTestPoint {
                    kind: "canvas_render_test".into(),
                    name: point.to_string(),
                    meta: CanvasRenderTestPointMeta {
                        time_used: format!("{:.3}", end - start),
                        data_content_hash: hash_bytes(&data_content),
                        text_content_hash: hash_bytes(&text_content),
                    },
                    verbose: {
                        let mut verbose_data = HashMap::new();
                        if cfg!(feature = "web_verbose") {
                            verbose_data.insert("data_content".into(), data_content);
                            verbose_data.insert("text_content".into(), text_content);
                        }
                        verbose_data
                    },
                })
                .unwrap()
                .into(),
                &"<<< typst_ts_test_capture".into(),
            );
            (end - start, artifact)
        };

        window
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&canvas)
            .unwrap();

        self::console_log!("canvas {point} {:.3}ms", serde_task.0);
    }

    #[wasm_bindgen_test]
    fn render_ir_test() {
        render_test_template("main_artifact_ir", MAIN_ARTIFACT_IR, "ir");
    }

    #[wasm_bindgen_test]
    fn render_json_test() {
        render_test_template("main_artifact_json", MAIN_ARTIFACT_JSON, "js");
    }
}
