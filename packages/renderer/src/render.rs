#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_test_common::embedded_artifact::*;
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

    async fn render_test_template(point: &str, artifact: &[u8], format: &str) {
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
                .await
                .unwrap();
            let end = performance.now();

            let data_content = canvas.to_data_url_with_type("image/png").unwrap();

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
    async fn render_ir_test() {
        render_test_template("main_artifact_ir", MAIN_ARTIFACT_IR, "ir").await;
    }

    #[wasm_bindgen_test]
    async fn render_json_test() {
        render_test_template("main_artifact_json", MAIN_ARTIFACT_JSON, "js").await;
    }

    #[wasm_bindgen_test]
    async fn render_line_1_ir_test() {
        render_test_template("line_1_artifact_ir", LINE_1_ARTIFACT_IR, "ir").await;
    }

    #[wasm_bindgen_test]
    async fn render_line_1_json_test() {
        render_test_template("line_1_artifact_json", LINE_1_ARTIFACT_JSON, "js").await;
    }

    #[wasm_bindgen_test]
    async fn render_line_2_ir_test() {
        render_test_template("line_2_artifact_ir", LINE_2_ARTIFACT_IR, "ir").await;
    }

    #[wasm_bindgen_test]
    async fn render_line_2_json_test() {
        render_test_template("line_2_artifact_json", LINE_2_ARTIFACT_JSON, "js").await;
    }

    #[wasm_bindgen_test]
    async fn render_path_1_ir_test() {
        render_test_template("path_1_artifact_ir", PATH_1_ARTIFACT_IR, "ir").await;
    }

    #[wasm_bindgen_test]
    async fn render_path_1_json_test() {
        render_test_template("path_1_artifact_json", PATH_1_ARTIFACT_JSON, "js").await;
    }

    #[wasm_bindgen_test]
    async fn render_polygon_1_ir_test() {
        render_test_template("polygon_1_artifact_ir", POLYGON_1_ARTIFACT_IR, "ir").await;
    }

    #[wasm_bindgen_test]
    async fn render_polygon_1_json_test() {
        render_test_template("polygon_1_artifact_json", POLYGON_1_ARTIFACT_JSON, "js").await;
    }
    #[wasm_bindgen_test]
    async fn render_shape_aspect() {
        render_test_template(
            "shape_aspect_1_artifact_ir",
            SHAPE_ASPECT_1_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_1_artifact_json",
            SHAPE_ASPECT_1_ARTIFACT_JSON,
            "js",
        )
        .await;
        render_test_template(
            "shape_aspect_2_artifact_ir",
            SHAPE_ASPECT_2_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_2_artifact_json",
            SHAPE_ASPECT_2_ARTIFACT_JSON,
            "js",
        )
        .await;
        render_test_template(
            "shape_aspect_3_artifact_ir",
            SHAPE_ASPECT_3_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_3_artifact_json",
            SHAPE_ASPECT_3_ARTIFACT_JSON,
            "js",
        )
        .await;
        render_test_template(
            "shape_aspect_4_artifact_ir",
            SHAPE_ASPECT_4_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_4_artifact_json",
            SHAPE_ASPECT_4_ARTIFACT_JSON,
            "js",
        )
        .await;
        render_test_template(
            "shape_aspect_5_artifact_ir",
            SHAPE_ASPECT_5_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_5_artifact_json",
            SHAPE_ASPECT_5_ARTIFACT_JSON,
            "js",
        )
        .await;
        render_test_template(
            "shape_aspect_6_artifact_ir",
            SHAPE_ASPECT_6_ARTIFACT_IR,
            "ir",
        )
        .await;
        render_test_template(
            "shape_aspect_6_artifact_json",
            SHAPE_ASPECT_6_ARTIFACT_JSON,
            "js",
        )
        .await;
    }

    // todo: get cjk font from remote server
    // #[wasm_bindgen_test]
    // async fn render_text_chinese_test() {
    //     render_test_template("text_chinese_artifact_ir", TEXT_CHINESE_ARTIFACT_IR, "ir").await;
    //     render_test_template(
    //         "text_chinese_artifact_json",
    //         TEXT_CHINESE_ARTIFACT_JSON.await,
    //         "js",
    //     );
    // }

    #[wasm_bindgen_test]
    async fn render_text_deco_test() {
        render_test_template("text_deco_1_artifact_ir", TEXT_DECO_1_ARTIFACT_IR, "ir").await;
        render_test_template("text_deco_1_artifact_json", TEXT_DECO_1_ARTIFACT_JSON, "js").await;
        render_test_template("text_deco_2_artifact_ir", TEXT_DECO_2_ARTIFACT_IR, "ir").await;
        render_test_template("text_deco_2_artifact_json", TEXT_DECO_2_ARTIFACT_JSON, "js").await;
        render_test_template("text_deco_3_artifact_ir", TEXT_DECO_3_ARTIFACT_IR, "ir").await;
        render_test_template("text_deco_3_artifact_json", TEXT_DECO_3_ARTIFACT_JSON, "js").await;
    }

    #[wasm_bindgen_test]
    async fn render_text_emoji_test() {
        // render_test_template("text_emoji_1_artifact_ir", TEXT_EMOJI_1_ARTIFACT_IR, "ir").await;
        // render_test_template(
        //     "text_emoji_1_artifact_json",
        //     TEXT_EMOJI_1_ARTIFACT_JSON,
        //     "js",
        // )
        // .await;
        render_test_template("text_emoji_2_artifact_ir", TEXT_EMOJI_2_ARTIFACT_IR, "ir").await;
        render_test_template(
            "text_emoji_2_artifact_json",
            TEXT_EMOJI_2_ARTIFACT_JSON,
            "js",
        )
        .await;
    }
}
