use std::sync::Arc;

use reflexo_typst::{TypstDocument, TypstPagedDocument};
use reflexo_typst2vec::incr::IncrDocServer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct IncrServer {
    inner: IncrDocServer,
}

impl Default for IncrServer {
    fn default() -> Self {
        let mut this = Self {
            inner: IncrDocServer::default(),
        };
        this.inner.set_should_attach_debug_info(true);
        this
    }
}

impl IncrServer {
    pub(crate) fn update(&mut self, doc: Arc<TypstPagedDocument>) -> Vec<u8> {
        // evicted by compiler
        // comemo::evict(30);

        self.inner.pack_delta(TypstDocument::Paged(doc))
    }
}

#[wasm_bindgen]
impl IncrServer {
    pub fn set_attach_debug_info(&mut self, attach: bool) {
        self.inner.set_should_attach_debug_info(attach);
    }

    pub fn current(&mut self) -> Option<Vec<u8>> {
        self.inner.pack_current()
    }

    pub fn reset(&mut self) {
        self.inner = IncrDocServer::default();
    }
}
