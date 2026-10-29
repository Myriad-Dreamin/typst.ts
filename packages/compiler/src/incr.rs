use std::sync::Arc;

use reflexo_typst::{BrowserCompilerFeat, TypstDocument, TypstPagedDocument, WorldComputeGraph};
use reflexo_typst2vec::incr::IncrDocServer;
use wasm_bindgen::prelude::*;

use crate::sync::SourceMappingState;

#[wasm_bindgen]
pub struct IncrServer {
    inner: IncrDocServer,
    source_mapping: Option<SourceMappingState>,
    mapping_revision: u32,
}

impl Default for IncrServer {
    fn default() -> Self {
        let mut this = Self {
            inner: IncrDocServer::default(),
            source_mapping: None,
            mapping_revision: 0,
        };
        this.inner.set_should_attach_debug_info(true);
        this
    }
}

impl IncrServer {
    pub(crate) fn update(
        &mut self,
        doc: Arc<TypstPagedDocument>,
        graph: Arc<WorldComputeGraph<BrowserCompilerFeat>>,
    ) -> Vec<u8> {
        // evicted by compiler
        // comemo::evict(30);

        let delta = self.inner.pack_delta(&TypstDocument::Paged(doc.clone()));
        self.mapping_revision = self.mapping_revision.wrapping_add(1).max(1);
        self.source_mapping = Some(SourceMappingState::new(graph, doc));
        delta
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

    /// Revision of the latest successful source/document mapping.
    #[wasm_bindgen(getter)]
    pub fn mapping_revision(&self) -> u32 {
        self.mapping_revision
    }

    /// Resolve a UTF-8 source byte offset to paged document positions.
    pub fn source_to_document(&self, path: String, byte_offset: u32) -> Result<JsValue, JsValue> {
        let positions = self
            .source_mapping
            .as_ref()
            .map(|mapping| mapping.source_to_document(&path, byte_offset as usize))
            .unwrap_or_default();
        serde_wasm_bindgen::to_value(&positions).map_err(|error| error.to_string().into())
    }

    /// Resolve a page-space point to a UTF-8 source byte offset.
    pub fn document_to_source(&self, page_offset: u32, x: f32, y: f32) -> Result<JsValue, JsValue> {
        let location = self
            .source_mapping
            .as_ref()
            .and_then(|mapping| mapping.document_to_source(page_offset as usize, x, y));
        serde_wasm_bindgen::to_value(&location).map_err(|error| error.to_string().into())
    }

    pub fn reset(&mut self) {
        self.inner = IncrDocServer::default();
        self.inner.set_should_attach_debug_info(true);
        self.source_mapping = None;
        self.mapping_revision = 0;
    }
}
