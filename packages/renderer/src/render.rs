#[cfg(test)]
// #[cfg(target_arch = "wasm32")]
mod tests {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_test_common::web_artifact::get_corpus;
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

    async fn get_ir_artifact(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.artifact.tir.bin", name))
            .await
            .unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    async fn get_json_artifact(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.artifact.json", name)).await.unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    async fn render_test_from_corpus(path: &str) {
        let point = path.replace('/', "_");
        let ir_point = format!("{}_artifact_ir", point);
        let json_point = format!("{}_artifact_json", point);

        render_test_template(&ir_point, &get_ir_artifact(path).await, "ir").await;
        render_test_template(&json_point, &get_json_artifact(path).await, "js").await;
    }

    macro_rules! make_test_point {
        ($name:ident, $($path:literal),+ $(,)?) => {
            #[wasm_bindgen_test]
            async fn $name() {
                $(
                    render_test_from_corpus($path).await;
                )*
            }
        };
    }

    make_test_point!(test_render_math_main, "math/main");

    make_test_point!(
        test_render_layout_clip,
        "layout/clip_1",
        "layout/clip_2",
        "layout/clip_3",
        "layout/clip_4",
    );
    make_test_point!(
        test_render_layout_list_marker,
        "layout/list_marker_1",
        "layout/list_marker_2",
        "layout/list_marker_3",
        "layout/list_marker_4",
    );
    make_test_point!(
        test_render_layout_transform,
        "layout/transform_1",
        "layout/transform_2",
        "layout/transform_3",
        "layout/transform_4",
    );

    make_test_point!(
        test_render_visual_line,
        "visualize/line_1",
        "visualize/line_2"
    );

    make_test_point!(test_render_visualize_path, "visualize/path_1");
    make_test_point!(test_render_visualize_polygon, "visualize/polygon_1");
    make_test_point!(
        test_render_visualize_shape_aspect,
        "visualize/shape_aspect_1",
        "visualize/shape_aspect_2",
        "visualize/shape_aspect_3",
        "visualize/shape_aspect_4",
        "visualize/shape_aspect_5",
        "visualize/shape_aspect_6",
    );
    make_test_point!(
        test_render_visualize_shape_circle,
        "visualize/shape_circle_1",
        "visualize/shape_circle_2",
        "visualize/shape_circle_3",
        "visualize/shape_circle_4",
    );

    // todo: get cjk font from remote server
    // make_test_point!(test_render_text_chinese, "text/chinese");

    make_test_point!(
        test_render_text_deco,
        "text/deco_1",
        "text/deco_2",
        "text/deco_3"
    );

    make_test_point!(
        test_render_text_emoji,
        // "text/emoji_1",
        "text/emoji_2"
    );
}
