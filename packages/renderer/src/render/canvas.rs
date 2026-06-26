use std::collections::HashMap;
use std::sync::OnceLock;

use reflexo_typst::error::prelude::*;
use reflexo_typst::hash::Fingerprint;
use reflexo_typst::vector::ir::{Axes, Rect, Scalar};
use reflexo_vec2canvas::{
    BrowserFontMetric, CanvasBound, CanvasDevice, DefaultExportFeature, ExportFeature,
};
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

    pub fn hit_canvas_page_bound(
        &mut self,
        ses: &RenderSession,
        page_off: usize,
        x: f32,
        y: f32,
    ) -> Result<JsValue> {
        let mut kern = ses.client.lock().unwrap();
        let mut client = ses.canvas_kern.lock().unwrap();

        let Some(bound) = client.hit_page_bound(&mut kern, page_off, x, y)? else {
            return Ok(JsValue::NULL);
        };

        canvas_bound_to_js(bound)
    }
}

static FONT_METRICS: OnceLock<BrowserFontMetric> = OnceLock::new();

fn canvas_bound_to_js(bound: CanvasBound) -> Result<JsValue> {
    let rect = js_sys::Object::new();
    set_js_number(&rect, "x", bound.rect.left().0 as f64)?;
    set_js_number(&rect, "y", bound.rect.top().0 as f64)?;
    set_js_number(&rect, "width", bound.rect.width().0 as f64)?;
    set_js_number(&rect, "height", bound.rect.height().0 as f64)?;

    let res = js_sys::Object::new();
    set_js_string(&res, "kind", bound.kind)?;
    js_sys::Reflect::set(&res, &"rect".into(), &rect)
        .map_err(map_into_err::<JsValue, _>("Renderer.SetCanvasBoundRect"))?;
    Ok(res.into())
}

fn set_js_number(obj: &js_sys::Object, key: &str, value: f64) -> Result<()> {
    js_sys::Reflect::set(obj, &key.into(), &value.into())
        .map_err(map_into_err::<JsValue, _>("Renderer.SetCanvasBoundNumber"))?;
    Ok(())
}

fn set_js_string(obj: &js_sys::Object, key: &str, value: &str) -> Result<()> {
    js_sys::Reflect::set(obj, &key.into(), &value.into())
        .map_err(map_into_err::<JsValue, _>("Renderer.SetCanvasBoundString"))?;
    Ok(())
}

impl TypstRenderer {
    #[allow(clippy::await_holding_lock)]
    pub async fn render_page_to_canvas_internal<Feat: ExportFeature>(
        &mut self,
        ses: &RenderSession,
        canvas: Option<&dyn CanvasDevice>,
        options: Option<RenderPageImageOptions>,
    ) -> Result<(Fingerprint, JsValue, Option<HashMap<String, f64>>)> {
        let opts = options.unwrap_or_default();
        let mut kern = ses.client.lock().unwrap();
        let mut client = ses.canvas_kern.lock().unwrap();

        let pixel_per_pt = opts.pixel_per_pt.or(ses.pixel_per_pt);
        client.set_pixel_per_pt(pixel_per_pt.unwrap_or(3.));
        let background_color = opts.background_color.as_deref();
        let background_color = background_color.or(ses.background_color.as_deref());
        client.set_fill(background_color.unwrap_or("ffffff").into());

        let should_render_body = opts.renders_canvas_body();
        let data_selection = opts.data_selection.unwrap_or(u32::MAX);
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
        let rect = opts.window_rect().unwrap_or(Rect {
            lo: Axes::new(Scalar(-1.), Scalar(-1.)),
            hi: Axes::new(Scalar(1e30), Scalar(1e30)),
        });
        let page = if let Some(page) = pages.get(page_num) {
            page.clone()
        } else {
            return Err(error_once!("Renderer.MissingPage", idx: page_num));
        };
        let fingerprint = page.content;

        if should_render_body {
            let cached = opts
                .cache_key
                .as_deref()
                .map(|c| c == fingerprint.as_svg_id("c"))
                .unwrap_or(false);

            let canvas = canvas.ok_or_else(|| error_once!("Renderer.MissingCanvasForBody"))?;

            if !cached {
                let prepare = client.prepare_page_resources(&mut kern, &[page_num])?;
                drop(client);
                drop(kern);
                if let Some(prepare) = prepare {
                    prepare.await;
                }

                kern = ses.client.lock().unwrap();
                client = ses.canvas_kern.lock().unwrap();
                let pixel_per_pt = opts.pixel_per_pt.or(ses.pixel_per_pt);
                client.set_pixel_per_pt(pixel_per_pt.unwrap_or(3.));
                let background_color = opts.background_color.as_deref();
                let background_color = background_color.or(ses.background_color.as_deref());
                client.set_fill(background_color.unwrap_or("ffffff").into());

                client
                    .render_page_in_window(&mut kern, canvas, page_num, rect)
                    .await?;
            }
        }

        if let Some(worker) = tc.as_mut() {
            let metric = FONT_METRICS.get_or_init(BrowserFontMetric::from_env);

            let mut output = vec![];
            let mut t = SemaTask::new(true, *metric, page.size.x.0, page.size.y.0);
            let ts = tiny_skia::Transform::identity();
            t.render_semantics(&kern.doc.module, ts, page.content, &mut output);
            worker.push(output.concat());
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
