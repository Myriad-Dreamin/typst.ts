/// <reference types="@vitest/browser/context" />

import { describe, expect, it } from 'vitest';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import { page, commands } from '@vitest/browser/context';

import { createTypstRenderer } from '../src/renderer.mjs';

// nodejs
const isNode =
    typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const fsImport = (file: string) => {
    const fs = require('fs');
    const path = require('path');
    return fs.readFileSync(path.join(import.meta.dirname, file));
};

const getModule = () => {
    const compiler = () => {
        throw new Error("shouldn't load compiler when testing renderer")
    };
    if (isNode) {
        return {
            compiler,
            renderer: () => fsImport('../../../../renderer/pkg/typst_ts_renderer_bg.wasm'),
        };
    }
    return {
        compiler,
        renderer: () => rendererUrl,
    };
};

const getRenderer = async () => {
    const r = createTypstRenderer();
    await r.init({ getModule: getModule().renderer });
    return r;
};

export const testSvg = async (data: Uint8Array) => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const rendered = await renderer.runWithSession(async renderSession => {
        renderer.manipulateData({
            renderSession,
            action: 'reset',
            data,
        });
        return await renderer.renderSvg({
            renderSession,
        });
    });
    container.innerHTML = rendered;
    const width = Number.parseFloat((container.firstElementChild as any).dataset.width);
    const height = Number.parseFloat((container.firstElementChild as any).dataset.height);
    page.viewport(width, height);
    document.body.appendChild(container);
    return { container, width, height };
};

export const testCanvas = async (data: Uint8Array) => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const { width, height } = await renderer.runWithSession(async renderSession => {
        renderer.manipulateData({
            renderSession,
            action: 'reset',
            data,
        });
        const width = renderSession.docWidth;
        const height = renderSession.docHeight;
        page.viewport(width, height);
        await renderer.renderToCanvas({
            renderSession,
            container,
        });

        return { width, height };
    });

    return { container, width, height };
};

export const makeSnapshot = async (s: { container: HTMLElement, width: number, height: number }, name: string) => {
    const snapshotPath = await page.screenshot({ save: true, path: `../../screenshots/renderer/${name}` });
    const { createSnapshot } = commands as any;
    const ret = await createSnapshot(snapshotPath, name, `size=${s.width}x${s.height}`);
    // screenshotHash, refHash
    expect(ret.screenshotHash).toEqual(ret.refHash);
};

// todo worker, text tests

// #[cfg(feature = "worker")]
// async fn render_in_worker_thread(
//     artifact: &[u8],
//     format: &str,
//     canvas: &web_sys::HtmlCanvasElement,
// ) -> (String, PerfMap) {
//     use std::sync::Arc;

//     use js_sys::Uint8Array;

//     let repo = "http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer";
//     let renderer_wrapper = format!("{repo}/pkg/typst_ts_renderer.mjs");
//     let renderer_wasm = format!("{repo}/pkg/typst_ts_renderer_bg.wasm");

//     let worker_script = r#"let renderer = null; let blobIdx = 0; let blobs = new Map();
// function recvMsgOrLoadSvg({data}) {
// if (data[0] && data[0].blobIdx) { console.log(data); let blobResolve = blobs.get(data[0].blobIdx); if (blobResolve) { blobResolve(data[1]); } return; }
// renderer.then(r => r.send(data)); }
// self.loadSvg = function (data, format, w, h) { return new Promise(resolve => {
// blobIdx += 1; blobs.set(blobIdx, resolve); postMessage({ exception: 'loadSvg', token: { blobIdx }, data, format, w, h }, { transfer: [ data.buffer ] });
// }); }

// onmessage = recvMsgOrLoadSvg; const m = import("http://localhost:20810/core/dist/esm/main.bundle.mjs"); const s = import({{renderer_wrapper}}); const w = fetch({{renderer_wasm}});
// renderer = m
// .then((m) => { const r = m.createTypstRenderer(); return r.init({ beforeBuild: [], getWrapper: () => s, getModule: () => w }).then(_ => r.workerBridge()); })"#.replace("{{renderer_wrapper}}",
// renderer_wrapper.as_str()
// ).replace("{{renderer_wasm}}", renderer_wasm.as_str());

//     let window = web_sys::window().expect("should have a window in this context");
//     let performance = window
//         .performance()
//         .expect("performance should be available");

//     let create = performance.now();

//     let renderer = WORKER.lock().unwrap();
//     let renderer = renderer.get_or_init(|| {
//         let tag = web_sys::BlobPropertyBag::new();
//         tag.set_type("application/javascript");

//         let parts = js_sys::Array::new();
//         parts.push(&Uint8Array::from(worker_script.as_bytes()).into());
//         let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &tag).unwrap();

//         let worker_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

//         let opts = web_sys::WorkerOptions::new();
//         opts.set_type(web_sys::WorkerType::Module);
//         let worker = web_sys::Worker::new_with_options(&worker_url, &opts).unwrap();

//         SendWrapper::new(Mutex::new(create_worker(worker)))
//     });
//     let renderer = &mut renderer.lock().unwrap();

//     let start = performance.now();
//     let session = renderer
//         .create_session(Some(CreateSessionOptions {
//             format: Some(format.to_string()),
//             artifact_content: Some(artifact.to_owned()),
//         }))
//         .await
//         .unwrap();
//     web_sys::console::log_1(&"session created".into());
//     session.set_background_color("#ffffff".to_string()).await;
//     session.set_pixel_per_pt(3.).await;

//     let sizes = &session.get_pages_info().await;
//     canvas.set_width((sizes.width() * 3.).ceil() as u32);
//     canvas.set_height((sizes.height() * 3.).ceil() as u32);

//     let prepare = performance.now();

//     let (_fingerprint, res, perf_events) = renderer
//         .render_page_to_canvas(Arc::new(session), Some(canvas), None)
//         .await
//         .unwrap();
//     let end = performance.now();

//     let text_content = js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();

//     let perf_events = perf_events.map(|mut p: HashMap<String, f64>| {
//         p.insert("create_renderer".to_string(), start - create);
//         p.insert("session_prepare".to_string(), prepare - start);
//         p.insert("rendering".to_string(), end - start);
//         p
//     });

//     (text_content, perf_events)
// }
