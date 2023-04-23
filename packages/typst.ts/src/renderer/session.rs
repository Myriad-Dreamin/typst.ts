use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RenderSession {
    pub(crate) pixel_per_pt: f32,
    pub(crate) background_color: String,
    pub(crate) doc: typst::doc::Document,
}

#[wasm_bindgen]
impl RenderSession {
    #[wasm_bindgen(getter)]
    pub fn pixel_per_pt(&self) -> f32 {
        self.pixel_per_pt.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn background_color(&self) -> String {
        self.background_color.clone()
    }
}

impl RenderSession {
    pub(crate) fn from_doc(doc: typst::doc::Document) -> Self {
        Self {
            pixel_per_pt: 0.,
            background_color: "".to_string(),
            doc,
        }
    }
}
