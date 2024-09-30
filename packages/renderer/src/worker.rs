use std::cell::OnceCell;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, sync::atomic::AtomicI32};

use js_sys::{Promise, Uint8Array};
use reflexo_typst::error::prelude::*;
use reflexo_typst::hash::Fingerprint;
use reflexo_typst2vec::stream::RkyvStreamData;
use reflexo_vec2canvas::DefaultExportFeature;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::ser::serializers::AllocSerializer;
use rkyv::{AlignedVec, Archive, Deserialize, Serialize};
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, HtmlImageElement, OffscreenCanvas};

use crate::TypstRenderer;
use crate::{
    session::{CreateSessionOptions, PagesInfo},
    RenderPageImageOptions, RenderSession,
};

type JsWorker = web_sys::Worker;
type RRenderSession = RemoteRenderSession;

const WORKER_SCRIPT: &str = r#"let renderer = null; let blobIdx = 0; let blobs = new Map();
function recvMsgOrLoadSvg({data}) { 
  if (data[0] && data[0].blobIdx) { console.log(data); let blobResolve = blobs.get(data[0].blobIdx); if (blobResolve) { blobResolve(data[1]); } return; }
  renderer.then(r => r.send(data)); }
self.loadSvg = function (data, format, w, h) { return new Promise(resolve => {
  blobIdx += 1; blobs.set(blobIdx, resolve); postMessage({ exception: 'loadSvg', token: { blobIdx }, data, format, w, h }, { transfer: [ data.buffer ] });
}); }
onmessage = function ({data}) { 
    onmessage = recvMsgOrLoadSvg; const m = import(data[2]); const s = import(data[4]); const w = fetch(data[6]);
    renderer = m
     .then((m) => { const r = m.createTypstRenderer(); return r.init({ beforeBuild: [], getWrapper: () => s, getModule: () => w }).then(_ => r.workerBridge()); })
}"#;

pub(crate) fn create_worker() -> Arc<Worker> {
    let tag = web_sys::BlobPropertyBag::new();
    tag.set_type("application/javascript");

    let parts = js_sys::Array::new();
    parts.push(&Uint8Array::from(WORKER_SCRIPT.as_bytes()).into());
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &tag).unwrap();

    let worker_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    // todo: we need to revoke it at appropriate time

    let opts = web_sys::WorkerOptions::new();
    opts.set_type(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options(&worker_url, &opts).unwrap();

    let repo = "http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer";
    let renderer_wrapper = format!("{repo}/pkg/typst_ts_renderer.mjs");
    let renderer_wasm = format!("{repo}/pkg/typst_ts_renderer_bg.wasm");
    let init_opts = [
        "init",
        "mainScript",
        "http://localhost:20810/core/dist/main.mjs",
        "rendererWrapper",
        renderer_wrapper.as_str(),
        "rendererWasm",
        renderer_wasm.as_str(),
    ];
    let msg = js_sys::Array::from_iter(init_opts.into_iter().map(JsValue::from_str));

    worker.post_message(&msg.into()).unwrap();

    #[allow(clippy::arc_with_non_send_sync)]
    let this = Arc::new(Worker {
        js: worker,
        request_idx: AtomicI32::new(0),
        requests: Mutex::new(HashMap::new()),
        _handler: OnceCell::new(),
    });

    let that = Arc::downgrade(&this);
    this._handler.get_or_init(|| {
        let handler = Closure::wrap(Box::new(move |event: JsValue| {
            let Some(that) = that.upgrade() else {
                web_sys::console::log_1(&"worker dropped when handling code".into());
                return;
            };

            // { exception: 'loadSvg', idx, blob }
            web_sys::console::log_1(&event);
            let data = js_sys::Reflect::get(&event, &JsValue::from_str("data")).unwrap();
            let resp = data.dyn_into::<js_sys::Array>();

            let resp = match resp {
                Ok(resp) => resp,
                Err(resp) => {
                    if let Ok(exception) =
                        js_sys::Reflect::get(&resp, &JsValue::from_str("exception"))
                    {
                        let exception = exception.as_string().expect("string exception");
                        if exception == "loadSvg" {
                            let token = js_sys::Reflect::get(&resp, &JsValue::from_str("token"))
                                .expect("token ?");
                            let data = js_sys::Reflect::get(&resp, &JsValue::from_str("data"))
                                .expect("data ?");
                            let format = js_sys::Reflect::get(&resp, &JsValue::from_str("format"))
                                .expect("format ?");

                            let blob = {
                                let parts = js_sys::Array::new();
                                parts.push(&data);

                                let tag = web_sys::BlobPropertyBag::new();
                                tag.set_type(&format.as_string().unwrap());
                                web_sys::Blob::new_with_u8_array_sequence_and_options(
                                    &parts,
                                    // todo: security check
                                    // https://security.stackexchange.com/questions/148507/how-to-prevent-xss-in-svg-file-upload
                                    // todo: use our custom font
                                    &tag,
                                )
                                .unwrap()
                            };

                            let that = Arc::clone(&that);

                            let img = HtmlImageElement::new().unwrap();
                            wasm_bindgen_futures::spawn_local(async move {
                                let p = Worker::exception_create_image_blob(&blob, &img);
                                p.await;
                                let canvas =
                                    web_sys::OffscreenCanvas::new(img.width(), img.height())
                                        .unwrap();

                                let ctx = canvas
                                    .get_context("2d")
                                    .expect("get context 2d")
                                    .expect("get context 2d");
                                let ctx = ctx
                                    .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
                                    .expect("must be OffscreenCanvasRenderingContext2d");
                                ctx.draw_image_with_html_image_element(&img, 0., 0.)
                                    .expect("must draw_image_with_html_image_element");
                                web_sys::console::log_1(&"owo6".into());

                                let image_data: JsValue = canvas
                                    .transfer_to_image_bitmap()
                                    .expect("transfer_to_image_bitmap")
                                    .into();
                                web_sys::console::log_1(&"owo7".into());

                                let res = js_sys::Array::from_iter([token, image_data.clone()]);
                                let transfer = js_sys::Array::from_iter([image_data]);

                                that.js
                                    .post_message_with_transfer(&res.into(), &transfer)
                                    .unwrap();
                                web_sys::console::log_1(&"owo5".into());
                            });

                            return;
                        }

                        web_sys::console::log_2(&"invalid exception found".into(), &event);
                        return;
                    }

                    web_sys::console::log_2(&"invalid response found".into(), &event);
                    return;
                }
            };

            let idx = resp.get(0).as_f64().unwrap() as i32;
            let resp = resp.get(1);

            let mut requests = that.requests.lock().unwrap();
            let Some(resolve) = requests.remove(&idx) else {
                web_sys::console::log_1(&format!("no request found for {idx}").into());
                return;
            };
            resolve.call1(&JsValue::NULL, &resp).unwrap();
        }) as Box<dyn FnMut(_)>);

        this.js
            .set_onmessage(Some(handler.as_ref().unchecked_ref()));
        handler
    });

    this
}

pub struct Worker {
    js: JsWorker,
    request_idx: AtomicI32,
    requests: Mutex<HashMap<i32, js_sys::Function>>,

    _handler: OnceCell<Closure<dyn FnMut(JsValue)>>,
}

impl Worker {
    fn pack(self: &Arc<Self>, req: Request) -> Msg {
        let idx = self
            .request_idx
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Msg {
            id: idx,
            request: req,
        }
    }

    fn send(self: &Arc<Self>, req: Request) -> Promise {
        let msg = self.pack(req);
        let idx = msg.id;
        self.js.post_message(&msg.to_bytes().into()).unwrap();

        let this = Arc::clone(self);
        let promise = Promise::new(&mut move |resolve, _reject| {
            this.requests.lock().unwrap().insert(idx, resolve);
        });

        promise
    }

    fn send_with(self: &Arc<Self>, req: Request, transfers: JsValue) -> Promise {
        let msg = self.pack(req);
        let idx = msg.id;
        let req = js_sys::Array::from_iter([msg.to_bytes().into(), transfers.clone()]);
        let transfers = js_sys::Array::from_iter([transfers]);
        self.js
            .post_message_with_transfer(&req.into(), &transfers.into())
            .unwrap();

        let this = Arc::clone(self);
        let promise = Promise::new(&mut move |resolve, _reject| {
            this.requests.lock().unwrap().insert(idx, resolve);
        });

        promise
    }

    async fn request(self: &Arc<Self>, req: Request) -> JsValue {
        JsFuture::from(self.send(req)).await.unwrap()
    }

    async fn request_with(self: &Arc<Self>, req: Request, transfers: JsValue) -> JsValue {
        JsFuture::from(self.send_with(req, transfers))
            .await
            .unwrap()
    }

    // session.set_background_color("#ffffff".to_string());
    // session.set_pixel_per_pt(3.);
    pub async fn create_session(
        self: &Arc<Self>,
        opts: Option<CreateSessionOptions>,
    ) -> ZResult<RRenderSession> {
        let req = Request::CreateSession(opts);
        let res = self.request(req).await;

        let session_info = res.as_f64().unwrap() as i32;
        Ok(RRenderSession {
            worker: Arc::clone(self),
            session_info,
        })
    }

    pub async fn render_page_to_canvas(
        self: &Arc<Self>,
        ses: &RemoteRenderSession,
        canvas: Option<&web_sys::HtmlCanvasElement>,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<(Fingerprint, JsValue, Option<HashMap<String, f64>>)> {
        let canvas = canvas.unwrap().transfer_control_to_offscreen().unwrap();
        let res = self
            .request_with(
                Request::RenderPageToCanvas(ses.session_info, options),
                canvas.into(),
            )
            .await;
        type P = (Fingerprint, String, Option<HashMap<String, f64>>);
        Ok(from_result::<P, _>(
            &res,
            |(fg, content, res): &<P as rkyv::Archive>::Archived, _| {
                let content = js_sys::JSON::parse(content.as_str()).unwrap();
                let mut dmap = SharedDeserializeMap::default();

                (
                    fg.deserialize(&mut dmap).unwrap(),
                    content,
                    res.deserialize(&mut dmap).unwrap(),
                )
            },
        ))
        // let text_content =
        // js_sys::JSON::stringify(&res).unwrap().as_string().unwrap();
    }

    async fn exception_create_image_blob(blob: &Blob, image_elem: &HtmlImageElement) {
        let data_url = web_sys::Url::create_object_url_with_blob(blob).unwrap();

        let img_load_promise = Promise::new(
            &mut move |complete: js_sys::Function, _reject: js_sys::Function| {
                let data_url = data_url.clone();
                let data_url2 = data_url.clone();
                let complete2 = complete.clone();

                image_elem.set_src(&data_url);

                // simulate async callback from another thread
                let a = Closure::<dyn Fn()>::new(move || {
                    web_sys::Url::revoke_object_url(&data_url).unwrap();
                    complete.call0(&complete).unwrap();
                });

                image_elem.set_onload(Some(a.as_ref().unchecked_ref()));
                a.forget();

                let a = Closure::<dyn Fn(JsValue)>::new(move |e: JsValue| {
                    web_sys::Url::revoke_object_url(&data_url2).unwrap();
                    complete2.call0(&complete2).unwrap();
                    // let end = std::time::Instant::now();
                    web_sys::console::log_1(
                        &format!(
                            "err image loading in {:?} {:?} {:?} {}",
                            // end - begin,
                            0,
                            js_sys::Reflect::get(&e, &"type".into()).unwrap(),
                            js_sys::JSON::stringify(&e).unwrap(),
                            data_url2,
                        )
                        .into(),
                    );
                });

                image_elem.set_onerror(Some(a.as_ref().unchecked_ref()));
                a.forget();
            },
        );

        wasm_bindgen_futures::JsFuture::from(img_load_promise)
            .await
            .unwrap();
    }
}

#[derive(Debug, Archive, Serialize, Deserialize)]
enum Request {
    CreateSession(Option<CreateSessionOptions>),
    RenderPageToCanvas(i32, Option<RenderPageImageOptions>),
    RemoveSession(i32),
    SetBackgroundColor(i32, String),
    SetPixelPerPt(i32, f32),
    GetPagesInfo(i32),
}

#[derive(Debug, Archive, Serialize, Deserialize)]
struct Msg {
    id: i32,
    request: Request,
}

impl Msg {
    fn to_bytes(&self) -> Vec<u8> {
        to_bytes(self)
    }
}

fn array_or_shared(js: JsValue) -> (Uint8Array, Option<js_sys::Array>) {
    match js.dyn_ref::<Uint8Array>() {
        Some(t) => (t.clone(), None),
        None => {
            let t = js
                .dyn_into::<js_sys::Array>()
                .expect("array_or_shared failed");
            (Uint8Array::new(&t.get(0)), Some(t))
        }
    }
}

fn to_bytes<T: rkyv::Serialize<AllocSerializer<0>>>(t: &T) -> Vec<u8> {
    // Or you can customize your serialization for better performance
    // and compatibility with #![no_std] environments
    use rkyv::ser::Serializer;

    let mut serializer = AllocSerializer::<0>::default();
    serializer.serialize_value(t).unwrap();
    let bytes = serializer.into_serializer().into_inner();

    bytes.into_vec()
}

fn from_result<P: rkyv::Archive + Sized, T>(
    js: &JsValue,
    f: impl FnOnce(&P::Archived, Option<js_sys::Array>) -> T,
) -> T {
    let (t, arr) = array_or_shared(js.clone());

    let sz = t.length() as usize;
    let mut buf = AlignedVec::with_capacity(sz);
    unsafe { buf.set_len(sz) };
    t.copy_to(buf.as_mut_slice());
    // copy_to
    let bs = RkyvStreamData::from(buf);
    let t = unsafe { bs.unchecked_peek::<P>() };

    f(t, arr)
}

#[derive(Default)]
#[wasm_bindgen]
pub struct WorkerBridge {
    pub(crate) plugin: TypstRenderer,
    pub(crate) sessions: HashMap<i32, RenderSession>,
    pub(crate) sesions: i32,
}

#[wasm_bindgen]
impl WorkerBridge {
    pub async fn send(&mut self, msg: JsValue) {
        let (x, y) = from_result::<Msg, _>(&msg, |x, y| {
            let mut dmap = SharedDeserializeMap::default();
            let x = x
                .deserialize(&mut dmap)
                .expect("WorkerBridge deserialize failed");

            (x, y)
        });
        web_sys::console::log_1(&format!("msg: {x:?}").into());
        let Msg { request: x, id } = x;

        let res = match x {
            Request::CreateSession(opts) => {
                let session = self.plugin.create_session(opts).unwrap();
                let idx = self.sesions;
                self.sesions += 1;
                self.sessions.insert(idx, session);
                JsValue::from_f64(idx as f64)
            }
            Request::RenderPageToCanvas(ses, opts) => {
                let y = y.unwrap();
                let session = self.sessions.get(&ses).unwrap();
                let canvas = y.get(1).dyn_into::<OffscreenCanvas>().unwrap();
                let ctx = canvas.get_context("2d").unwrap().unwrap();
                let ctx = ctx
                    .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
                    .unwrap();
                let (fg, content, res) = self
                    .plugin
                    .render_page_to_canvas_internal::<DefaultExportFeature>(
                        session,
                        Some(&ctx),
                        opts,
                    )
                    .await
                    .unwrap();
                let content = js_sys::JSON::stringify(&content)
                    .unwrap()
                    .as_string()
                    .unwrap();
                let p = (fg, content, res);
                let b = to_bytes(&p);
                JsValue::from(Uint8Array::from(&b[..]))
            }

            Request::RemoveSession(ses) => {
                self.sessions.remove(&ses);
                JsValue::NULL
            }

            Request::SetBackgroundColor(ses, color) => {
                let session = self.sessions.get_mut(&ses).unwrap();
                session.set_background_color(color);
                JsValue::NULL
            }

            Request::SetPixelPerPt(ses, f) => {
                let session = self.sessions.get_mut(&ses).unwrap();
                session.set_pixel_per_pt(f);
                JsValue::NULL
            }

            Request::GetPagesInfo(ses) => {
                let session = self.sessions.get_mut(&ses).unwrap();
                let info = session.pages_info();
                let b = to_bytes(&info);
                JsValue::from(Uint8Array::from(&b[..]))
            }
        };

        let resp = js_sys::Array::from_iter([JsValue::from_f64(id as f64), res]);

        let global = js_sys::global();
        let ws = global
            .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
            .unwrap();
        ws.post_message(&resp).unwrap();
    }
}

pub struct RemoteRenderSession {
    worker: Arc<Worker>,
    session_info: i32,
}

impl Drop for RemoteRenderSession {
    fn drop(&mut self) {
        web_sys::console::log_1(&"dropped RemoteRenderSession".into());
        let this = Arc::clone(&self.worker);
        let session_info = self.session_info;
        wasm_bindgen_futures::spawn_local(async move {
            let res = this.request(Request::RemoveSession(session_info)).await;
            web_sys::console::log_2(&"remove session result:".into(), &res);
        });
    }
}

impl RemoteRenderSession {
    pub async fn set_background_color(&self, to_string: String) {
        self.worker
            .request(Request::SetBackgroundColor(self.session_info, to_string))
            .await;
    }

    pub async fn set_pixel_per_pt(&self, f: f32) {
        self.worker
            .request(Request::SetPixelPerPt(self.session_info, f))
            .await;
    }

    pub async fn get_pages_info(&self) -> PagesInfo {
        let res = self
            .worker
            .request(Request::GetPagesInfo(self.session_info))
            .await;
        from_result::<PagesInfo, _>(&res, |x, _| {
            let mut dmap = SharedDeserializeMap::default();
            x.deserialize(&mut dmap).unwrap()
        })
    }
}
