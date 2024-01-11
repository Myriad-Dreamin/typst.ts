use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

use crate::{RenderSession, TypstRenderer};

#[wasm_bindgen]
impl TypstRenderer {
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

        dom_kern
            .mount(&mut kern, elem, tiny_skia::Rect::from_xywh(x, y, w, h))
            .await
            .unwrap();
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

        dom_kern
            .rerender(
                &mut kern,
                tiny_skia::Rect::from_xywh(x, y, w, h),
                is_responsive,
            )
            .await
            .unwrap();
    }
}
