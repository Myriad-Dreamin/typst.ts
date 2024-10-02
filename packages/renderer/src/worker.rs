#![allow(clippy::await_holding_lock)]

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
use web_sys::{HtmlCanvasElement, HtmlImageElement, OffscreenCanvas};

use crate::TypstRenderer;
use crate::{
    session::{CreateSessionOptions, PagesInfo},
    RenderPageImageOptions, RenderSession,
};

#[wasm_bindgen]
impl TypstRenderer {
    pub async fn create_worker(&mut self, w: web_sys::Worker) -> ZResult<TypstWorker> {
        let core = create_worker(w);
        #[allow(clippy::arc_with_non_send_sync)]
        let rs = Arc::new(core.create_session(None).await?);
        Ok(TypstWorker { core, rs })
    }

    pub fn create_worker_bridge(self) -> ZResult<WorkerBridge> {
        Ok(WorkerBridge {
            plugin: self,
            ..Default::default()
        })
    }
}

#[repr(u8)]
enum CanvasAction {
    Delete,
    New,
    Update,
}

impl TryFrom<u8> for CanvasAction {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == CanvasAction::Delete as u8 => Ok(CanvasAction::Delete),
            x if x == CanvasAction::New as u8 => Ok(CanvasAction::New),
            x if x == CanvasAction::Update as u8 => Ok(CanvasAction::Update),
            _ => Err(()),
        }
    }
}

#[wasm_bindgen]
pub struct TypstWorker {
    core: Arc<WorkerCore>,
    // todo: multiple sessions
    rs: Arc<RemoteRenderSession>,
}

#[wasm_bindgen]
impl TypstWorker {
    pub fn manipulate_data(&mut self, action: &str, data: Uint8Array) -> ZResult<Promise> {
        let resp = self.core.send_with(
            Request::ManipulateData(self.rs.session_info, action.to_string()),
            data.into(),
        );

        Ok(resp)
    }

    pub fn get_pages_info(&self) -> Promise {
        let rs = self.rs.clone();
        wasm_bindgen_futures::future_to_promise(async move { Ok(rs.get_pages_info().await.into()) })
    }

    pub fn render_canvas(
        &mut self,
        actions: Vec<u8>,
        canvas_list: Vec<HtmlCanvasElement>,
        data: Vec<RenderPageImageOptions>,
    ) -> ZResult<Promise> {
        if actions.len() != data.len() || canvas_list.len() != data.len() {
            return Err(error_once!("Renderer.InvalidActionDataLength"));
        }

        web_sys::console::log_1(&format!("render_canvas {data:?}").into());

        let mut promises = Vec::new();
        let inp = actions.into_iter().zip(canvas_list).zip(data);
        for ((action, canvas), data) in inp {
            let action = CanvasAction::try_from(action)
                .map_err(|_| error_once!("Renderer.InvalidAction", action: action as u32))?;

            match action {
                CanvasAction::Delete => {
                    promises.push(wasm_bindgen_futures::future_to_promise(async move {
                        Ok(JsValue::NULL)
                    }));
                }
                CanvasAction::New => {
                    let canvas = canvas.transfer_control_to_offscreen().unwrap();
                    let w = self.rs.worker.clone();
                    let p =
                        w.render_page_to_canvas_internal(self.rs.clone(), Some(canvas), Some(data));
                    promises.push(wasm_bindgen_futures::future_to_promise(async move {
                        let (fingerprint, html_semantics, ..) = p.await?;

                        let res = js_sys::Object::new();
                        let err = js_sys::Reflect::set(
                            &res,
                            &"cacheKey".into(),
                            &fingerprint.as_svg_id("c").into(),
                        );
                        err.map_err(map_into_err::<JsValue, _>("Renderer.SetCacheKey"))?;
                        let err =
                            js_sys::Reflect::set(&res, &"htmlSemantics".into(), &html_semantics);
                        err.map_err(map_into_err::<JsValue, _>("Renderer.SetHtmlSemantics"))?;
                        Ok(res.into())
                    }));
                }
                CanvasAction::Update => {
                    let w = self.rs.worker.clone();
                    let p = w.render_page_to_canvas_internal(self.rs.clone(), None, Some(data));
                    promises.push(wasm_bindgen_futures::future_to_promise(async move {
                        let (fingerprint, html_semantics, ..) = p.await?;

                        let res = js_sys::Object::new();
                        let err = js_sys::Reflect::set(
                            &res,
                            &"cacheKey".into(),
                            &fingerprint.as_svg_id("c").into(),
                        );
                        err.map_err(map_into_err::<JsValue, _>("Renderer.SetCacheKey"))?;
                        let err =
                            js_sys::Reflect::set(&res, &"htmlSemantics".into(), &html_semantics);
                        err.map_err(map_into_err::<JsValue, _>("Renderer.SetHtmlSemantics"))?;
                        Ok(res.into())
                    }));
                }
            }
        }

        Ok(Promise::all(&js_sys::Array::from_iter(promises).into()))
    }
}

type JsWorker = web_sys::Worker;
type RRenderSession = RemoteRenderSession;

pub(crate) fn create_worker(js: JsWorker) -> Arc<WorkerCore> {
    #[allow(clippy::arc_with_non_send_sync)]
    let this = Arc::new(WorkerCore {
        js,
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
                                let p =
                                    reflexo_vec2canvas::exception_create_image_blob(&blob, &img);
                                p.await;
                                let image_data: JsValue =
                                    reflexo_vec2canvas::html_image_to_bitmap(&img).into();

                                let res = js_sys::Array::from_iter([token, image_data.clone()]);
                                let transfer = js_sys::Array::from_iter([image_data]);

                                that.js
                                    .post_message_with_transfer(&res.into(), &transfer)
                                    .unwrap();
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
            drop(requests);
            resolve.call1(&JsValue::NULL, &resp).unwrap();
        }) as Box<dyn FnMut(_)>);

        this.js
            .set_onmessage(Some(handler.as_ref().unchecked_ref()));
        handler
    });

    this
}

#[wasm_bindgen]
pub struct WorkerCore {
    js: JsWorker,
    request_idx: AtomicI32,
    requests: Mutex<HashMap<i32, js_sys::Function>>,

    _handler: OnceCell<Closure<dyn FnMut(JsValue)>>,
}

impl WorkerCore {
    fn pack(self: &Arc<Self>, req: Request) -> Msg {
        let idx = self
            .request_idx
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Msg {
            id: idx,
            request: req,
        }
    }

    fn send(self: Arc<Self>, req: Request) -> Promise {
        let msg = self.pack(req);
        let idx = msg.id;
        self.js.post_message(&msg.to_bytes().into()).unwrap();

        let promise = Promise::new(&mut move |resolve, _reject| {
            self.requests.lock().unwrap().insert(idx, resolve);
        });

        promise
    }

    fn send_with(self: &Arc<Self>, req: Request, transfers: JsValue) -> Promise {
        let msg = self.pack(req);
        let idx = msg.id;
        let body = Uint8Array::from(msg.to_bytes().as_slice());
        let buf = body.buffer().into();
        let req = js_sys::Array::from_iter([body.into(), transfers.clone()]);
        let transfers = if let Some(t) = transfers.dyn_ref::<js_sys::Uint8Array>() {
            t.buffer().into()
        } else {
            transfers
        };
        web_sys::console::log_2(&"send_with".into(), &transfers);
        let transfers = if transfers == JsValue::UNDEFINED {
            js_sys::Array::from_iter([buf])
        } else {
            js_sys::Array::from_iter([buf, transfers])
        };
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
        JsFuture::from(self.clone().send(req)).await.unwrap()
    }

    async fn request_with(self: &Arc<Self>, req: Request, transfers: JsValue) -> JsValue {
        JsFuture::from(self.send_with(req, transfers))
            .await
            .unwrap()
    }

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
        ses: Arc<RemoteRenderSession>,
        canvas: Option<&web_sys::HtmlCanvasElement>,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<(Fingerprint, JsValue, Option<HashMap<String, f64>>)> {
        let canvas = canvas.map(|x| x.transfer_control_to_offscreen().unwrap());
        self.clone()
            .render_page_to_canvas_internal(ses.clone(), canvas, options)
            .await
    }

    pub async fn render_page_to_canvas_internal(
        self: Arc<Self>,
        ses: Arc<RemoteRenderSession>,
        canvas: Option<web_sys::OffscreenCanvas>,
        options: Option<RenderPageImageOptions>,
    ) -> ZResult<(Fingerprint, JsValue, Option<HashMap<String, f64>>)> {
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
}

#[derive(Debug, Archive, Serialize, Deserialize)]
enum Request {
    CreateSession(Option<CreateSessionOptions>),
    ManipulateData(i32, String),
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
    // todo: desync
    pub(crate) sessions: HashMap<i32, Arc<Mutex<RenderSession>>>,
    pub(crate) canvases: HashMap<i32, HashMap<usize, OffscreenCanvas>>,
    pub(crate) previous_promise: Option<Promise>,
    pub(crate) sesions: i32,
}

#[wasm_bindgen]
impl WorkerBridge {
    pub fn send(&mut self, msg: JsValue) {
        let (x, y) = from_result::<Msg, _>(&msg, |x, y| {
            let mut dmap = SharedDeserializeMap::default();
            let x = x
                .deserialize(&mut dmap)
                .expect("WorkerBridge deserialize failed");

            (x, y)
        });
        web_sys::console::log_1(&format!("msg: {x:?}").into());
        let Msg { request: x, id } = x;

        let previous_promise = self.previous_promise.take();
        let res = match x {
            Request::CreateSession(opts) => {
                let session = self.plugin.create_session(opts).unwrap();
                let idx = self.sesions;
                self.sesions += 1;
                #[allow(clippy::arc_with_non_send_sync)]
                self.sessions.insert(idx, Arc::new(Mutex::new(session)));
                Ok(JsValue::from_f64(idx as f64))
            }
            Request::ManipulateData(ses, action) => {
                let session = self.sessions.get_mut(&ses).unwrap().clone();
                let data = y.unwrap();
                let data = data.get(1).dyn_into::<Uint8Array>().unwrap().to_vec();
                let p = wasm_bindgen_futures::future_to_promise(async move {
                    let res = match action.as_str() {
                        "reset" => session.lock().unwrap().reset_current(&data),
                        "merge" => session.lock().unwrap().merge_delta(&data),
                        _ => Err(error_once!("Renderer.UnsupportedAction", action: action)),
                    };
                    if let Err(e) = res {
                        web_sys::console::log_1(&format!("manipulate error: {e}").into());
                    }
                    Ok(JsValue::NULL)
                });
                Err(p)
            }
            Request::RenderPageToCanvas(ses, opts) => {
                let y = y.unwrap();
                let session = self.sessions.get(&ses).unwrap().clone();
                let canvas = y.get(1);
                let canvases = self.canvases.entry(ses).or_default();
                let canvas = match canvas {
                    _ if canvas == JsValue::UNDEFINED => canvases
                        .get(&opts.as_ref().unwrap().page_off)
                        .unwrap()
                        .clone(),
                    canvas => {
                        let canvas = canvas.dyn_into::<OffscreenCanvas>().unwrap();
                        canvases.insert(opts.as_ref().unwrap().page_off, canvas.clone());
                        canvas
                    }
                };
                // canvases

                let ctx = canvas.get_context("2d").unwrap().unwrap();
                let ctx = ctx
                    .dyn_into::<web_sys::OffscreenCanvasRenderingContext2d>()
                    .unwrap();
                let mut plugin = self.plugin.clone();
                let magic = js_sys::Math::random();
                let previous_promise = previous_promise.clone();
                let p = wasm_bindgen_futures::future_to_promise(async move {
                    if let Some(p) = previous_promise {
                        web_sys::console::log_1(&"wait for previous promise 2".into());
                        let _ = wasm_bindgen_futures::JsFuture::from(p).await;
                        web_sys::console::log_1(&"wait for previous promise 2 end".into());
                    }
                    web_sys::console::log_1(&format!("render_page_to_canvas lock {magic}").into());
                    let ses = session.lock().unwrap();
                    web_sys::console::log_1(&"render_page_to_canvas lock 2".into());
                    let (fg, content, res) = plugin
                        .render_page_to_canvas_internal::<DefaultExportFeature>(
                            &ses,
                            Some(&ctx),
                            opts,
                        )
                        .await
                        .unwrap();
                    drop(ses);
                    web_sys::console::log_1(&"render_page_to_canvas lock end".into());
                    let content = if content == JsValue::UNDEFINED {
                        JsValue::NULL
                    } else {
                        content
                    };
                    let t = js_sys::JSON::stringify(&content).unwrap();
                    web_sys::console::log_3(
                        &"js_sys::JSON::stringify".into(),
                        &content,
                        &(&t).into(),
                    );
                    let content: String = t.into();
                    let p = (fg, content, res);
                    let b = to_bytes(&p);
                    Ok(JsValue::from(Uint8Array::from(&b[..])))
                });
                Err(p)
            }

            Request::RemoveSession(ses) => {
                self.sessions.remove(&ses);
                Ok(JsValue::NULL)
            }

            Request::SetBackgroundColor(ses, color) => {
                let session = self.sessions.get(&ses).unwrap().clone();
                let p = wasm_bindgen_futures::future_to_promise(async move {
                    session.lock().unwrap().set_background_color(color);
                    Ok(JsValue::NULL)
                });
                Err(p)
            }

            Request::SetPixelPerPt(ses, f) => {
                let session = self.sessions.get(&ses).unwrap().clone();
                let p = wasm_bindgen_futures::future_to_promise(async move {
                    session.lock().unwrap().set_pixel_per_pt(f);
                    Ok(JsValue::NULL)
                });
                Err(p)
            }

            Request::GetPagesInfo(ses) => {
                let session = self.sessions.get(&ses).unwrap().clone();
                let p = wasm_bindgen_futures::future_to_promise(async move {
                    let info = session.lock().unwrap().pages_info();
                    let b = to_bytes(&info);
                    Ok(JsValue::from(Uint8Array::from(&b[..])))
                });
                Err(p)
            }
        };

        let p = wasm_bindgen_futures::future_to_promise(async move {
            if let Some(p) = previous_promise {
                web_sys::console::log_1(&"wait for previous promise".into());
                let _ = wasm_bindgen_futures::JsFuture::from(p).await;
                web_sys::console::log_1(&"wait for previous promise end".into());
            }

            let res = res;
            let id = id;
            let res = match res {
                Ok(res) => res,
                Err(e) => wasm_bindgen_futures::JsFuture::from(e).await.unwrap(),
            };

            let resp = js_sys::Array::from_iter([JsValue::from_f64(id as f64), res]);

            let global = js_sys::global();
            let ws = global
                .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
                .unwrap();
            ws.post_message(&resp).unwrap();
            web_sys::console::log_1(&"post_message end".into());
            Ok(JsValue::NULL)
        });
        self.previous_promise = Some(p.clone());

        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(p).await;
        });
    }
}

pub struct RemoteRenderSession {
    worker: Arc<WorkerCore>,
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
