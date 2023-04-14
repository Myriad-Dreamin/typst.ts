// use append_only_vec::AppendOnlyVec;
use comemo::Prehashed;
use typst::{
    font::{Font, FontBook, FontInfo},
    syntax::SourceId,
    util::Buffer,
};
use typst_ts_core::{font::BufferFontLoader, FontResolver, FontSlot};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

use crate::web_font::WebFont;

/// A world that provides access to the browser.
pub struct TypstBrowserWorld {
    book: Prehashed<FontBook>,
    fonts: Vec<FontSlot>,
    pub main: SourceId,
}

impl TypstBrowserWorld {
    // todo: better parameter type
    pub async fn new(searcher: BrowserFontSearcher) -> Result<Self, JsValue> {
        Ok(Self {
            // library: Prehashed::new(typst_library::build()),
            book: Prehashed::new(searcher.book),
            fonts: searcher.fonts,
            main: SourceId::detached(),
        })
    }

    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get()
    }
}

impl FontResolver for TypstBrowserWorld {
    fn font_book(&self) -> &FontBook {
        &self.book
    }

    fn get_font(&self, idx: usize) -> Font {
        self.font(idx).unwrap()
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

        // self.fonts.push(FontSlot {
        //     index: 0 as u32,
        //     font: (
        //         OnceCell::new(),
        //         FontLoadProvider::new(Box::new(BufferFontLoader {
        //             buffer: Some(buffer),
        //             index: 0 as u32,
        //         })),
        //     ),
        // });

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
