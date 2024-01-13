use std::{cell::RefCell, rc::Rc};

use typst_ts_dom_exporter::RenderTask;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;

use crate::{RenderSession, TypstRenderer};

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
#[wasm_bindgen]
impl TypstRenderer {
    #[allow(clippy::await_holding_lock)]
    fn post_schedule_dom(ses: &mut RenderSession, task: RenderTask, is_responsive: bool) {
        web_sys::console::log_1(&format!("dom task: {:?}", task.transact()).into());
        if !task.is_finished() {
            let client = ses.client.clone();
            let dom_kern = ses.dom_kern.clone();

            // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
            // number of times. After it's done we want all our resources cleaned up. To
            // achieve this we're using an `Rc`. The `Rc` will eventually store the
            // closure we want to execute on each frame, but to start out it contains
            // `None`.
            //
            // After the `Rc` is made we'll actually create the closure, and the closure
            // will reference one of the `Rc` instances. The other `Rc` reference is
            // used to store the closure, request the first frame, and then is dropped
            // by this function.
            //
            // Inside the closure we've got a persistent `Rc` reference, which we use
            // for all future iterations of the loop
            let h = Rc::new(RefCell::new((None, task)));
            let g = h.clone();

            g.borrow_mut().0 = Some(Closure::new(move || {
                let f = h.clone();
                let client = client.clone();
                let dom_kern = dom_kern.clone();
                spawn_local(async move {
                    let task = f.borrow().1.clone();
                    if task.is_finished() {
                        // Drop our handle to this closure so that it will get cleaned
                        // up once we return.
                        let _ = f.borrow_mut().0.take();

                        return;
                    }

                    let mut kern = client.lock().unwrap();
                    let mut dom_kern = dom_kern.lock().unwrap();

                    f.borrow_mut().1 = dom_kern
                        .reschedule(&mut kern, task, is_responsive)
                        .await
                        .unwrap();

                    // Schedule ourself for another requestAnimationFrame callback.
                    request_animation_frame(f.borrow().0.as_ref().unwrap());
                });
            }));

            request_animation_frame(g.borrow().0.as_ref().unwrap());
        }
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn mount_dom(
        &mut self,
        ses: &mut RenderSession,
        elem: HtmlElement,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let mut kern = ses.client.lock().unwrap();
        let mut dom_kern = ses.dom_kern.lock().unwrap();

        let task = dom_kern
            .mount(&mut kern, elem, tiny_skia::Rect::from_xywh(x, y, w, h))
            .await
            .unwrap();

        drop(dom_kern);
        drop(kern);

        Self::post_schedule_dom(ses, task, false);
    }

    #[allow(clippy::await_holding_lock)]
    pub async fn trigger_dom_rerender(
        &mut self,
        ses: &mut RenderSession,
        feature: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) {
        let mut kern = ses.client.lock().unwrap();
        let mut dom_kern = ses.dom_kern.lock().unwrap();

        let is_responsive = (feature & (1 << 0)) != 0;

        let task = dom_kern
            .rerender(
                &mut kern,
                tiny_skia::Rect::from_xywh(x, y, w, h),
                is_responsive,
            )
            .await
            .unwrap();

        drop(dom_kern);
        drop(kern);

        Self::post_schedule_dom(ses, task, true);
    }
}
