use std::sync::OnceLock;
use std::{collections::HashMap, ops::Deref};

use reflexo_typst::error::prelude::*;
use reflexo_typst::hash::Fingerprint;
use reflexo_typst::vector::ir::{Axes, LayoutRegionNode, Rect, Scalar};
use reflexo_vec2canvas::{BrowserFontMetric, CanvasDevice, DefaultExportFeature, ExportFeature};
use reflexo_vec2sema::SemaTask;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, OffscreenCanvasRenderingContext2d};

use crate::{RenderPageImageOptions, RenderSession, TypstRenderer};

#[wasm_bindgen]
impl TypstRenderer {
    pub async fn render_page_to_canvas(
        &mut self,
        ses: &RenderSession,
        canvas: JsValue,
        options: Option<RenderPageImageOptions>,
    ) -> Result<JsValue> {
        let canvas = canvas.as_ref();
        let canvas = if canvas == &JsValue::NULL {
            None
        } else {
            Some(match canvas.dyn_ref::<CanvasRenderingContext2d>() {
                Some(t) => t as &dyn CanvasDevice,
                None => canvas
                    .dyn_ref::<OffscreenCanvasRenderingContext2d>()
                    .unwrap() as &dyn CanvasDevice,
            })
        };

        let (fingerprint, html_semantics, ..) = self
            .render_page_to_canvas_internal::<DefaultExportFeature>(ses, canvas, options)
            .await?;

        let res = js_sys::Object::new();
        let err =
            js_sys::Reflect::set(&res, &"cacheKey".into(), &fingerprint.as_svg_id("c").into());
        err.map_err(map_into_err::<JsValue, _>("Renderer.SetCacheKey"))?;
        let err = js_sys::Reflect::set(&res, &"htmlSemantics".into(), &html_semantics);
        err.map_err(map_into_err::<JsValue, _>("Renderer.SetHtmlSemantics"))?;
        Ok(res.into())
    }
}

static FONT_METRICS: OnceLock<BrowserFontMetric> = OnceLock::new();

impl TypstRenderer {
    #[allow(clippy::await_holding_lock)]
    pub async fn render_page_to_canvas_internal<Feat: ExportFeature>(
        &mut self,
        ses: &RenderSession,
        canvas: Option<&dyn CanvasDevice>,
        options: Option<RenderPageImageOptions>,
    ) -> Result<(Fingerprint, JsValue, Option<HashMap<String, f64>>)> {
        let opts = options.unwrap_or_default();
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

        let pixel_per_pt = opts.pixel_per_pt.or(ses.pixel_per_pt);
        client.set_pixel_per_pt(pixel_per_pt.unwrap_or(3.));
        let background_color = opts.background_color.as_deref();
        let background_color = background_color.or(ses.background_color.as_deref());
        client.set_fill(background_color.unwrap_or("ffffff").into());

        let data_selection = opts.data_selection.unwrap_or(u32::MAX);

        let should_render_body = (data_selection & (1 << 0)) != 0;
        // semantics layer
        let mut tc = ((data_selection & (1 << 3)) != 0).then(Vec::new);

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

        let page_num = opts.page_off;
        let fingerprint = if let Some(page) = pages.get(page_num) {
            page.content
        } else {
            return Err(error_once!("Renderer.MissingPage", idx: page_num));
        };

        if should_render_body {
            let cached = opts
                .cache_key
                .map(|c| c == fingerprint.as_svg_id("c"))
                .unwrap_or(false);

            let canvas = canvas.ok_or_else(|| error_once!("Renderer.MissingCanvasForBody"))?;

            if !cached {
                client
                    .render_page_in_window(&mut kern, canvas, page_num, rect)
                    .await?;
            }
        }

        // todo: leaking abstraction
        // todo: reuse
        if let Some(t) = &kern.layout {
            let pages = match t {
                LayoutRegionNode::Pages(a) => {
                    let (_, pages) = a.deref();
                    pages
                }
                _ => todo!(),
            };
            for (idx, page) in pages.iter().enumerate() {
                if page_num != idx {
                    continue;
                }
                if let Some(worker) = tc.as_mut() {
                    let metric = FONT_METRICS.get_or_init(BrowserFontMetric::from_env);

                    let mut output = vec![];
                    let mut t = SemaTask::new(true, *metric, page.size.x.0, page.size.y.0);
                    let ts = tiny_skia::Transform::identity();
                    t.render_semantics(&kern.doc.module, ts, page.content, &mut output);
                    worker.push(output.concat());
                }
            }
        }

        Ok((
            fingerprint,
            serde_wasm_bindgen::to_value(&tc)
                .map_err(map_into_err::<JsValue, _>("Renderer.EncodeHtmlSemantics"))?,
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

    use std::{
        collections::HashMap,
        sync::{Mutex, OnceLock},
    };

    use reflexo_vec2canvas::ExportFeature;
    use send_wrapper::SendWrapper;
    use serde::{Deserialize, Serialize};
    use sha2::Digest;
    use typst_ts_test_common::web_artifact::get_corpus;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_test::*;

    #[cfg(feature = "worker")]
    use crate::worker::{create_worker, WorkerCore};
    use crate::{session::CreateSessionOptions, TypstRenderer};

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    const SHOW_RESULT: bool = true;
    #[cfg(feature = "worker")]
    const IN_WORKER: bool = false;

    fn hash_bytes<T: AsRef<[u8]>>(bytes: T) -> String {
        format!("sha256:{}", hex::encode(sha2::Sha256::digest(bytes)))
    }

    #[derive(Serialize, Deserialize)]
    struct CanvasRenderTestPointMeta {
        time_used: String,
        data_content_hash: String,
        text_content_hash: String,
        artifact_hash: String,
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

    static RENDERER: Mutex<OnceLock<SendWrapper<Mutex<TypstRenderer>>>> =
        Mutex::new(OnceLock::new());

    #[cfg(feature = "worker")]
    static WORKER: Mutex<OnceLock<SendWrapper<Mutex<std::sync::Arc<WorkerCore>>>>> =
        Mutex::new(OnceLock::new());

    type PerfMap = Option<HashMap<String, f64>>;

    async fn render_in_main_thread(
        artifact: &[u8],
        format: &str,
        canvas: &web_sys::HtmlCanvasElement,
    ) -> (String, PerfMap) {
        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

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

        let (_fingerprint, res, perf_events) = renderer
            .render_page_to_canvas_internal::<CIRenderFeature>(&session, Some(&context), None)
            .await
            .unwrap();
        let end = performance.now();

        let text_content = js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();

        let perf_events = perf_events.map(|mut p| {
            p.insert("create_renderer".to_string(), start - create);
            p.insert("session_prepare".to_string(), prepare - start);
            p.insert("rendering".to_string(), end - start);
            p
        });

        (text_content, perf_events)
    }

    #[cfg(feature = "worker")]
    async fn render_in_worker_thread(
        artifact: &[u8],
        format: &str,
        canvas: &web_sys::HtmlCanvasElement,
    ) -> (String, PerfMap) {
        use std::sync::Arc;

        use js_sys::Uint8Array;

        let repo = "http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer";
        let renderer_wrapper = format!("{repo}/pkg/typst_ts_renderer.mjs");
        let renderer_wasm = format!("{repo}/pkg/typst_ts_renderer_bg.wasm");

        let worker_script = r#"let renderer = null; let blobIdx = 0; let blobs = new Map();
function recvMsgOrLoadSvg({data}) { 
    if (data[0] && data[0].blobIdx) { console.log(data); let blobResolve = blobs.get(data[0].blobIdx); if (blobResolve) { blobResolve(data[1]); } return; }
    renderer.then(r => r.send(data)); }
self.loadSvg = function (data, format, w, h) { return new Promise(resolve => {
    blobIdx += 1; blobs.set(blobIdx, resolve); postMessage({ exception: 'loadSvg', token: { blobIdx }, data, format, w, h }, { transfer: [ data.buffer ] });
}); }

onmessage = recvMsgOrLoadSvg; const m = import("http://localhost:20810/core/dist/esm/main.bundle.mjs"); const s = import({{renderer_wrapper}}); const w = fetch({{renderer_wasm}});
renderer = m
    .then((m) => { const r = m.createTypstRenderer(); return r.init({ beforeBuild: [], getWrapper: () => s, getModule: () => w }).then(_ => r.workerBridge()); })"#.replace("{{renderer_wrapper}}",
renderer_wrapper.as_str()
).replace("{{renderer_wasm}}", renderer_wasm.as_str());

        let window = web_sys::window().expect("should have a window in this context");
        let performance = window
            .performance()
            .expect("performance should be available");

        let create = performance.now();

        let renderer = WORKER.lock().unwrap();
        let renderer = renderer.get_or_init(|| {
            let tag = web_sys::BlobPropertyBag::new();
            tag.set_type("application/javascript");

            let parts = js_sys::Array::new();
            parts.push(&Uint8Array::from(worker_script.as_bytes()).into());
            let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &tag).unwrap();

            let worker_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

            let opts = web_sys::WorkerOptions::new();
            opts.set_type(web_sys::WorkerType::Module);
            let worker = web_sys::Worker::new_with_options(&worker_url, &opts).unwrap();

            SendWrapper::new(Mutex::new(create_worker(worker)))
        });
        let renderer = &mut renderer.lock().unwrap();

        let start = performance.now();
        let session = renderer
            .create_session(Some(CreateSessionOptions {
                format: Some(format.to_string()),
                artifact_content: Some(artifact.to_owned()),
            }))
            .await
            .unwrap();
        web_sys::console::log_1(&"session created".into());
        session.set_background_color("#ffffff".to_string()).await;
        session.set_pixel_per_pt(3.).await;

        let sizes = &session.get_pages_info().await;
        canvas.set_width((sizes.width() * 3.).ceil() as u32);
        canvas.set_height((sizes.height() * 3.).ceil() as u32);

        let prepare = performance.now();

        let (_fingerprint, res, perf_events) = renderer
            .render_page_to_canvas(Arc::new(session), Some(canvas), None)
            .await
            .unwrap();
        let end = performance.now();

        let text_content = js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();

        let perf_events = perf_events.map(|mut p: HashMap<String, f64>| {
            p.insert("create_renderer".to_string(), start - create);
            p.insert("session_prepare".to_string(), prepare - start);
            p.insert("rendering".to_string(), end - start);
            p
        });

        (text_content, perf_events)
    }

    async fn render_test_template(point: &str, artifact: &[u8], format: &str) {
        super::FONT_METRICS.get_or_init(super::BrowserFontMetric::new_test);

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
            let start = performance.now();

            #[cfg(feature = "worker")]
            let (text_content, perf_events) = if IN_WORKER {
                render_in_worker_thread(artifact, format, &canvas).await
            } else {
                render_in_main_thread(artifact, format, &canvas).await
            };
            #[cfg(not(feature = "worker"))]
            let (text_content, perf_events) =
                render_in_main_thread(artifact, format, &canvas).await;

            let end = performance.now();

            let data_content = canvas.to_data_url_with_type("image/png").unwrap();

            let data_content_hash = hash_bytes(&data_content);

            web_sys::console::log_3(
                &">>> reflexo_test_capture".into(),
                &serde_json::to_string(&CanvasRenderTestPoint {
                    kind: "canvas_render_test".into(),
                    name: point.to_string(),
                    meta: CanvasRenderTestPointMeta {
                        time_used: format!("{:.3}", end - start),
                        data_content_hash: data_content_hash.clone(),
                        text_content_hash: hash_bytes(&text_content),
                        artifact_hash: hash_bytes(artifact),
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
                &"<<< reflexo_test_capture".into(),
            );
            (end - start, perf_events, data_content_hash, artifact)
        };

        if SHOW_RESULT {
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
        }

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

        let artifact = get_ir_artifact(path).await;
        render_test_template(&ir_point, &artifact, "vector").await;
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
    // make_test_point!(test_render_math_undergradmath, "math/undergradmath");

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

    // make_test_point!(
    //     test_render_text_emoji,
    //     // "text/emoji_00",
    //     "text/emoji_01"
    // );
}
