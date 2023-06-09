use js_sys::Uint8Array;
use typst_ts_core::error::prelude::*;
use wasm_bindgen::prelude::*;

use crate::{RenderSession, TypstRenderer};

#[wasm_bindgen]
impl TypstRenderer {
    pub fn render_to_pdf(&mut self, artifact_content: &[u8]) -> ZResult<Uint8Array> {
        let session = self.session_from_artifact(artifact_content)?;
        self.render_to_pdf_in_session(&session)
    }

    pub fn render_to_pdf_in_session(&mut self, session: &RenderSession) -> ZResult<Uint8Array> {
        Ok(Uint8Array::from(
            self.render_to_pdf_internal(session)?.as_slice(),
        ))
    }
}

impl TypstRenderer {
    pub fn render_to_pdf_internal(&self, _session: &RenderSession) -> ZResult<Vec<u8>> {
        // contribution 510KB
        // Ok(typst::export::pdf(&session.doc))
        Err(error_once!("Renderer.PdfFeatureNotEnabled"))
    }
}
