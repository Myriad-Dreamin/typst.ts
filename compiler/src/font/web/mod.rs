use std::sync::{Arc, RwLock};

use js_sys::Promise;
use typst::{
    font::{Font, FontBook, FontInfo},
    util::Buffer,
};
use typst_ts_core::{
    font::{BufferFontLoader, FontProfile, FontResolverImpl, PartialFontBook},
    FontLoader, FontSlot, ReadAllOnce,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::Blob;

#[derive(Clone, Debug)]
pub struct WebFont {
    pub family: String,
    pub style: String,
    pub full_name: String,
    pub postscript_name: String,
    pub context: JsValue,
    pub blob: js_sys::Function,
}

impl WebFont {
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

pub struct WebFontBlob {
    font: Option<WebFont>,
}

impl WebFontBlob {
    pub fn new(font: WebFont) -> Self {
        Self { font: Some(font) }
    }
}

impl ReadAllOnce for WebFontBlob {
    fn read_all(mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let blob = pollster::block_on(self.font.take().unwrap().load());
        let blob = pollster::block_on(JsFuture::from(blob.array_buffer())).unwrap();
        let mut blob = js_sys::Uint8Array::new(&blob).to_vec();
        let blob_len = blob.len();
        buf.append(&mut blob);
        Ok(blob_len)
    }
}

/// Searches for fonts.
pub struct BrowserFontSearcher {
    pub book: FontBook,
    pub fonts: Vec<FontSlot>,
    pub profile: FontProfile,
    pub partial_book: Arc<RwLock<PartialFontBook>>,
}

impl BrowserFontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        let profile = FontProfile {
            version: "v1beta".to_owned(),
            ..Default::default()
        };
        Self {
            book: FontBook::new(),
            fonts: vec![],
            profile,
            partial_book: Arc::new(RwLock::new(PartialFontBook::default())),
        }
    }

    pub async fn add_web_fonts(&mut self, _fonts: js_sys::Array) {
        // family: String,
        // style: String,
        // full_name: String,
        // postscript_name: String,
        // context: JsValue,
        // blob: js_sys::Function,
        // let blob = font.load().await;
        // let blob = JsFuture::from(blob.array_buffer()).await.unwrap();
        // let buffer = Buffer::from(js_sys::Uint8Array::new(&blob).to_vec());

        // for f in fonts.iter() {
        //     // todo: load lazily
        //     self.fonts.push(FontSlot::new(Box::new(WebFontLoader {
        //         font: Some(f.dyn_into::<WebFont>().unwrap()),
        //         index: 0 as u32,
        //     })))
        // }
    }

    pub fn add_font_data(&mut self, buffer: Buffer) {
        for (i, info) in FontInfo::iter(buffer.as_slice()).enumerate() {
            self.book.push(info);

            let buffer = buffer.clone();
            self.fonts.push(FontSlot::new(Box::new(BufferFontLoader {
                buffer: Some(buffer),
                index: i as u32,
            })))
        }
    }
}

impl Default for BrowserFontSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BrowserFontSearcher> for FontResolverImpl {
    fn from(value: BrowserFontSearcher) -> Self {
        FontResolverImpl::new(value.book, value.partial_book, value.fonts, value.profile)
    }
}
