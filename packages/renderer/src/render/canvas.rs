use std::{collections::HashMap, hash::Hash, ops::Deref};

use reflexo_vec2canvas::{
    AnnotationListTask, DefaultExportFeature, ExportFeature, TextContentTask,
};
use typst_ts_core::{
    annotation::AnnotationList,
    error::prelude::*,
    hash::{Fingerprint, FingerprintHasher, FingerprintSipHasher},
    vector::ir::{Axes, LayoutRegionNode, Rect, Scalar},
    TextContent,
};
use wasm_bindgen::prelude::*;

use crate::{RenderPageImageOptions, RenderSession, TypstRenderer};

#[derive(Default)]
pub struct CanvasDataSelection {
    pub body: bool,
    pub text_content: bool,
    pub annotation_list: bool,
}

#[wasm_bindgen]
impl TypstRenderer {
    pub async fn render_page_to_canvas(
        &mut self,
        ses: &RenderSession,
        canvas: Option<web_sys::CanvasRenderingContext2d>,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<JsValue> {
        let (fingerprint, text_content, annotation_list, ..) = self
            .render_page_to_canvas_internal::<DefaultExportFeature>(ses, canvas, options)
            .await?;

        let res = js_sys::Object::new();
        let err =
            js_sys::Reflect::set(&res, &"cacheKey".into(), &fingerprint.as_svg_id("c").into());
        err.map_err(map_into_err::<JsValue, _>("Renderer.SetCacheKey"))?;
        let err = js_sys::Reflect::set(&res, &"textContent".into(), &text_content);
        err.map_err(map_into_err::<JsValue, _>("Renderer.SetTextContent"))?;
        let err = js_sys::Reflect::set(&res, &"annotationList".into(), &annotation_list);
        err.map_err(map_into_err::<JsValue, _>("Renderer.SetAnnotationContent"))?;
        Ok(res.into())
    }
}

impl TypstRenderer {
    #[allow(clippy::await_holding_lock)]
    pub async fn render_page_to_canvas_internal<Feat: ExportFeature>(
        &mut self,
        ses: &RenderSession,
        canvas: Option<web_sys::CanvasRenderingContext2d>,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<(Fingerprint, JsValue, JsValue, Option<HashMap<String, f64>>)> {
        let rect_lo_x: f32 = -1.;
        let rect_lo_y: f32 = -1.;
        let rect_hi_x: f32 = 1e30;
        let rect_hi_y: f32 = 1e30;
        let rect = Rect {
            lo: Axes::new(Scalar(rect_lo_x), Scalar(rect_lo_y)),
            hi: Axes::new(Scalar(rect_hi_x), Scalar(rect_hi_y)),
        };

        let mut kern = ses.client.lock().unwrap();
        let mut client = ses.canvas_kern.lock().unwrap();
        client.set_pixel_per_pt(ses.pixel_per_pt.unwrap_or(3.));
        client.set_fill(ses.background_color.as_deref().unwrap_or("ffffff").into());

        let data_selection = options
            .as_ref()
            .and_then(|o| o.data_selection)
            .unwrap_or(u32::MAX);

        let should_render_body = (data_selection & (1 << 0)) != 0;
        let mut tc = ((data_selection & (1 << 1)) != 0).then(TextContent::default);
        let mut annotations = ((data_selection & (1 << 2)) != 0).then(AnnotationList::default);

        // let def_provider = GlyphProvider::new(FontGlyphProvider::default());
        // let partial_providier =
        //     PartialFontGlyphProvider::new(def_provider,
        // self.session_mgr.font_resolver.clone());

        // worker.set_glyph_provider(GlyphProvider::new(partial_providier));

        // crate::utils::console_log!("use partial font glyph provider");

        let perf_events = if Feat::ENABLE_TRACING {
            Some(elsa::FrozenMap::<&'static str, Box<f64>>::default())
        } else {
            None
        };
        // if let Some(perf_events) = perf_events.as_ref() {
        //     worker.set_perf_events(perf_events)
        // };

        // todo: reuse
        let Some(t) = &kern.layout else {
            todo!();
        };
        let pages = t.pages(kern.module()).unwrap().pages();

        let (page_num, fingerprint) = if let Some(RenderPageImageOptions {
            page_off: Some(c),
            ..
        }) = options
        {
            (Some(c), pages[c].content)
        } else {
            let mut f = FingerprintSipHasher::default();
            for page in pages.iter() {
                page.content.hash(&mut f);
            }
            (None, f.finish_fingerprint().0)
        };

        if should_render_body {
            let cached = options
                .and_then(|o| o.cache_key)
                .map(|c| c == fingerprint.as_svg_id("c"))
                .unwrap_or(false);

            let canvas = &canvas.ok_or_else(|| error_once!("Renderer.MissingCanvasForBody"))?;

            if !cached {
                if let Some(page_num) = page_num {
                    client
                        .render_page_in_window(&mut kern, canvas, page_num, rect)
                        .await?;
                } else {
                    client.render_in_window(&mut kern, canvas, rect).await;
                }
            }
        }

        // todo: leaking abstraction
        let mut worker = tc
            .as_mut()
            .map(|tc| TextContentTask::new(&kern.doc.module, tc));
        let mut annotation_list_worker = annotations
            .as_mut()
            .map(|annotations| AnnotationListTask::new(&kern.doc.module, annotations));
        // todo: reuse
        if let Some(t) = &kern.layout {
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            let mut page_off = 0.;
            for (idx, page) in pages.iter().enumerate() {
                if page_num.map_or(false, |p| p != idx) {
                    page_off += page.size.y.0;
                    continue;
                }
                let partial_page_off = if page_num.is_some() { 0. } else { page_off };
                if let Some(worker) = worker.as_mut() {
                    worker.page_height = partial_page_off + page.size.y.0;
                    worker.process_flat_item(
                        tiny_skia::Transform::from_translate(partial_page_off, 0.),
                        &page.content,
                    );
                }
                if let Some(worker) = annotation_list_worker.as_mut() {
                    worker.page_num = idx as u32;
                    worker.process_flat_item(
                        tiny_skia::Transform::from_translate(partial_page_off, 0.),
                        &page.content,
                    );
                }
                page_off += page.size.y.0;
            }
        }

        Ok((
            fingerprint,
            serde_wasm_bindgen::to_value(&tc)
                .map_err(map_into_err::<JsValue, _>("Renderer.EncodeTextContent"))?,
            serde_wasm_bindgen::to_value(&annotations).map_err(map_into_err::<JsValue, _>(
                "Renderer.EncodeAnnotationContent",
            ))?,
            perf_events.map(|perf_events| {
                perf_events
                    .into_map()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect()
            }),
        ))
    }
}

#[cfg(test)]
#[cfg(target_arch = "wasm32")]
mod tests {
    #![allow(clippy::await_holding_lock)]

    use std::{collections::HashMap, sync::Mutex};

    use reflexo_vec2canvas::ExportFeature;
    use send_wrapper::SendWrapper;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    // use typst_ts_test_common::std_artifact::STD_TEST_FILES;
    use typst_ts_test_common::web_artifact::get_corpus;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;
    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    use crate::{session::CreateSessionOptions, TypstRenderer};

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

    pub struct CIRenderFeature;

    impl ExportFeature for CIRenderFeature {
        const ENABLE_TRACING: bool = true;
        const SHOULD_RENDER_TEXT_ELEMENT: bool = true;
    }

    static RENDERER: Mutex<once_cell::sync::OnceCell<SendWrapper<Mutex<TypstRenderer>>>> =
        Mutex::new(once_cell::sync::OnceCell::new());

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

        let (time_used, perf_events, data_content_hash, ..) = {
            let create = performance.now();

            let renderer = RENDERER.lock().unwrap();
            let renderer =
                renderer.get_or_init(|| SendWrapper::new(Mutex::new(crate::tests::get_renderer())));
            let renderer = &mut renderer.lock().unwrap();

            let start = performance.now();
            let mut session = renderer
                .create_session(Some(CreateSessionOptions {
                    format: Some(format.to_string()),
                    artifact_content: Some(artifact.to_owned()),
                }))
                .unwrap();
            session.set_background_color("#ffffff".to_string());
            session.set_pixel_per_pt(3.);

            let sizes = &session.pages_info;
            canvas.set_width((sizes.width() * 3.).ceil() as u32);
            canvas.set_height((sizes.height() * 3.).ceil() as u32);

            let context: web_sys::CanvasRenderingContext2d = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();

            let prepare = performance.now();

            let (_fingerprint, res, _, perf_events) = renderer
                .render_page_to_canvas_internal::<CIRenderFeature>(&session, Some(context), None)
                .await
                .unwrap();
            let end = performance.now();

            let data_content = canvas.to_data_url_with_type("image/png").unwrap();

            let text_content = js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();

            let data_content_hash = hash_bytes(&data_content);

            let settle = performance.now();

            let perf_events = perf_events.map(|mut p| {
                p.insert("create_renderer".to_string(), start - create);
                p.insert("session_prepare".to_string(), prepare - start);
                p.insert("rendering".to_string(), end - start);
                p.insert("serialize_result".to_string(), settle - end);
                p
            });

            web_sys::console::log_3(
                &">>> typst_ts_test_capture".into(),
                &serde_json::to_string(&CanvasRenderTestPoint {
                    kind: "canvas_render_test".into(),
                    name: point.to_string(),
                    meta: CanvasRenderTestPointMeta {
                        time_used: format!("{:.3}", end - start),
                        data_content_hash: data_content_hash.clone(),
                        text_content_hash: hash_bytes(&text_content),
                    },
                    verbose: {
                        let mut verbose_data = HashMap::new();
                        if cfg!(feature = "web_verbose") {
                            verbose_data.insert("data_content".into(), data_content);
                            verbose_data.insert("text_content".into(), text_content);
                            verbose_data.insert(
                                "perf_events".into(),
                                serde_json::to_string(&perf_events).unwrap(),
                            );
                        }
                        verbose_data
                    },
                })
                .unwrap()
                .into(),
                &"<<< typst_ts_test_capture".into(),
            );
            (end - start, perf_events, data_content_hash, artifact)
        };

        let div = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        div.set_attribute("style", "display block; border: 1px solid #000;")
            .unwrap();

        let title = window
            .document()
            .unwrap()
            .create_element("div")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();

        title.set_inner_html(&format!(
            "{point} => {data_content_hash} {time_used:.3}ms",
            point = point,
            data_content_hash = data_content_hash,
            time_used = time_used,
        ));

        div.append_child(&title).unwrap();
        div.append_child(&canvas).unwrap();

        window
            .document()
            .unwrap()
            .body()
            .unwrap()
            .append_child(&div)
            .unwrap();

        let perf_events = perf_events
            .as_ref()
            .map(|p| serde_wasm_bindgen::to_value(&p).unwrap());
        web_sys::console::log_2(
            &format!("canvas {point} => {data_content_hash} {time_used:.3}ms").into(),
            &perf_events.into(),
        );
    }

    async fn get_ir_artifact(name: &str) -> Vec<u8> {
        let array_buffer = get_corpus(format!("{}.artifact.sir.in", name))
            .await
            .unwrap();
        js_sys::Uint8Array::new(&array_buffer).to_vec()
    }

    async fn render_test_from_corpus(path: &str) {
        let point = path.replace('/', "_");
        let ir_point = format!("{}_artifact_ir", point);

        render_test_template(&ir_point, &get_ir_artifact(path).await, "vector").await;
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
    make_test_point!(test_render_math_undergradmath, "math/undergradmath");

    make_test_point!(
        test_render_layout_clip,
        "layout/clip_00",
        "layout/clip_01",
        "layout/clip_02",
        "layout/clip_03",
    );
    make_test_point!(
        test_render_layout_list_marker,
        "layout/list-marker_00",
        "layout/list-marker_01",
        "layout/list-marker_02",
        "layout/list-marker_03",
        "layout/list-marker_04",
    );
    make_test_point!(
        test_render_layout_transform,
        "layout/transform_00",
        "layout/transform_01",
        "layout/transform_02",
        "layout/transform_03",
    );

    make_test_point!(
        test_render_visual_line,
        "visualize/line_00",
        "visualize/line_01",
        "visualize/line_02",
        "visualize/line_03"
    );

    make_test_point!(
        test_render_visualize_path,
        "visualize/path_00",
        "visualize/path_01",
        "visualize/path_02",
        "visualize/path_03"
    );
    make_test_point!(
        test_render_visualize_polygon,
        "visualize/polygon_00",
        "visualize/polygon_01"
    );
    make_test_point!(
        test_render_visualize_shape_aspect,
        "visualize/shape-aspect_00",
        "visualize/shape-aspect_01",
        "visualize/shape-aspect_02",
        "visualize/shape-aspect_03",
        "visualize/shape-aspect_04",
        "visualize/shape-aspect_05",
        "visualize/shape-aspect_06",
    );
    make_test_point!(
        test_render_visualize_shape_circle,
        "visualize/shape-circle_00",
        "visualize/shape-circle_01",
        "visualize/shape-circle_02",
        "visualize/shape-circle_03",
        "visualize/shape-circle_04",
    );
    make_test_point!(
        test_render_visualize_stroke,
        "visualize/stroke_00",
        "visualize/stroke_01",
        "visualize/stroke_02",
        "visualize/stroke_03",
        "visualize/stroke_04",
        "visualize/stroke_05",
        "visualize/stroke_06",
        "visualize/stroke_07",
    );

    // todo: This will use font from local machine, which is unstable
    // make_test_point!(test_render_visualize_svg_text, "visualize/svg_text");

    // todo: get cjk font from remote server
    // make_test_point!(test_render_text_chinese, "text/chinese");

    make_test_point!(
        test_render_text_deco,
        "text/deco_00",
        "text/deco_01",
        "text/deco_02"
    );

    make_test_point!(
        test_render_text_emoji,
        // "text/emoji_00",
        "text/emoji_01"
    );
}
