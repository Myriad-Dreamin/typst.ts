use typst::geom::Abs;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone)]
pub struct PageInfo {
    pub(crate) width: Abs,
    pub(crate) height: Abs,
}

#[wasm_bindgen]
impl PageInfo {
    #[wasm_bindgen(getter)]
    pub fn width_pt(&self) -> f64 {
        self.width.to_pt()
    }

    #[wasm_bindgen(getter)]
    pub fn height_pt(&self) -> f64 {
        self.height.to_pt()
    }
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct PagesInfo {
    pub(crate) pages: Vec<PageInfo>,
}

#[wasm_bindgen]
impl PagesInfo {
    #[wasm_bindgen(getter)]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn page(&self, i: usize) -> PageInfo {
        self.pages[i].clone()
    }
}

#[wasm_bindgen]
pub struct RenderSession {
    pub(crate) pixel_per_pt: f32,
    pub(crate) background_color: String,
    pub(crate) doc: typst::doc::Document,
    pub(crate) pages_info: PagesInfo,
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

    #[wasm_bindgen(getter)]
    pub fn pages_info(&self) -> PagesInfo {
        self.pages_info.clone()
    }
}

impl RenderSession {
    pub(crate) fn from_doc(doc: typst::doc::Document) -> Self {
        let pages_info = PagesInfo {
            pages: {
                let mut pages = Vec::new();
                pages.reserve(doc.pages.len());
                for page in doc.pages.iter() {
                    pages.push(PageInfo {
                        width: page.size().x,
                        height: page.size().y,
                    });
                }
                pages
            },
        };

        Self {
            pixel_per_pt: 0.,
            background_color: "".to_string(),
            doc,
            pages_info,
        }
    }
}
