use typst_ts_dom_exporter::IncrDomDocClient;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

use crate::{RenderSession, TypstRenderer};

#[wasm_bindgen]
impl TypstRenderer {
    pub async fn mount_dom(
        &mut self,
        ses: &mut RenderSession,
        elem: HtmlElement,
    ) -> IncrDomDocClient {
        let mut dom_kern = IncrDomDocClient::default();
        dom_kern.set_client(ses.client.clone());
        dom_kern.mount(elem).await.unwrap();
        dom_kern
    }
}
