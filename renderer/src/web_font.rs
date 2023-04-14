use js_sys::Promise;
use typst::{font::Font, util::Buffer};
use typst_ts_core::FontLoader;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

#[wasm_bindgen(module = "fonts")]
extern "C" {
    fn load_font(font_ref: i32) -> Blob;
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WebFont {
    family: String,
    style: String,
    full_name: String,
    postscript_name: String,
    context: JsValue,
    blob: js_sys::Function,
}

#[wasm_bindgen]
impl WebFont {
    #[wasm_bindgen(getter)]
    pub fn family(&self) -> String {
        self.family.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_family(&mut self, family: String) {
        self.family = family;
    }

    #[wasm_bindgen(getter)]
    pub fn style(&self) -> String {
        self.style.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_style(&mut self, style: String) {
        self.style = style;
    }

    #[wasm_bindgen(getter)]
    pub fn full_name(&self) -> String {
        self.full_name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_full_name(&mut self, full_name: String) {
        self.full_name = full_name;
    }

    #[wasm_bindgen(getter)]
    pub fn postscript_name(&self) -> String {
        self.postscript_name.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_postscrip_name(&mut self, postscript_name: String) {
        self.postscript_name = postscript_name;
    }

    #[wasm_bindgen(getter)]
    pub fn context(&self) -> JsValue {
        self.context.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_context(&mut self, context: JsValue) {
        self.context = context;
    }

    #[wasm_bindgen(getter)]
    pub fn blob(&self) -> js_sys::Function {
        self.blob.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_blob(&mut self, blob: js_sys::Function) {
        self.blob = blob;
    }

    pub async fn load(&self) -> Blob {
        JsFuture::from(
            self.blob
                .call0(&self.context)
                .unwrap()
                .dyn_into::<Promise>()
                .unwrap(),
        )
        .await
        .unwrap()
        .into()
    }
}

pub struct WebFontLoader {
    font: Option<WebFont>,
    index: u32,
}

impl WebFontLoader {
    pub fn new(font: WebFont, index: u32) -> Self {
        Self {
            font: Some(font),
            index,
        }
    }
}

impl FontLoader for WebFontLoader {
    fn load(&mut self) -> Option<Font> {
        let blob = pollster::block_on(self.font.take().unwrap().load());
        let blob = pollster::block_on(JsFuture::from(blob.array_buffer())).unwrap();
        let blob = Buffer::from(js_sys::Uint8Array::new(&blob).to_vec());

        Font::new(blob, self.index)
    }
}
