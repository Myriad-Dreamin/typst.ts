// use append_only_vec::AppendOnlyVec;
use typst::{
    font::{FontBook, FontInfo},
    syntax::SourceId,
    util::Buffer,
};
use typst_ts_core::{font::BufferFontLoader, font::FontResolverImpl, FontSlot};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::web_font::WebFont;

/// A world that provides access to the browser.
pub struct TypstBrowserWorld {
    pub font_resolver: FontResolverImpl,
    pub main: SourceId,
}

impl TypstBrowserWorld {
    // todo: better parameter type
    pub async fn new(searcher: BrowserFontSearcher) -> Result<Self, JsValue> {
        // Hook up the lang items.
        // todo: bad upstream changes
        let library = typst_library::build();
        typst::eval::set_lang_items(library.items.clone());

        Ok(Self {
            // library: Prehashed::new(typst_library::build()),
            font_resolver: FontResolverImpl::new(searcher.book, searcher.fonts),
            main: SourceId::detached(),
        })
    }
}

/// Searches for fonts.
pub struct BrowserFontSearcher {
    pub book: FontBook,
    fonts: Vec<FontSlot>,
}

impl BrowserFontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    pub async fn add_web_font(&mut self, font: WebFont) {
        let blob = font.load().await;
        let blob = JsFuture::from(blob.array_buffer()).await.unwrap();
        let buffer = Buffer::from(js_sys::Uint8Array::new(&blob).to_vec());

        // todo: load lazily
        self.add_font_data(buffer);
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
