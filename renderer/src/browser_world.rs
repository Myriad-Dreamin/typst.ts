// use append_only_vec::AppendOnlyVec;
use comemo::Prehashed;
use once_cell::sync::OnceCell;
use typst::{
    font::{Font, FontBook, FontInfo},
    syntax::SourceId,
    util::Buffer,
};
use typst_ts_core::FontResolver;

use wasm_bindgen::prelude::*;

/// Holds details about the location of a font and lazily the font itself.
struct FontSlot {
    pub buffer: Buffer,
    pub index: u32,
    pub font: OnceCell<Option<Font>>,
}

/// A world that provides access to the browser.
pub struct TypstBrowserWorld {
    book: Prehashed<FontBook>,
    fonts: Vec<FontSlot>,
    pub main: SourceId,
}

impl TypstBrowserWorld {
    pub async fn new() -> Result<Self, JsValue> {
        let mut searcher = BrowserFontSearcher::new();

        // todo: receive font files from user
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/LinLibertine_R.ttf"
        )));
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/LinLibertine_RB.ttf"
        )));
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/LinLibertine_RBI.ttf"
        )));
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/LinLibertine_RI.ttf"
        )));
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/NewCMMath-Book.otf"
        )));
        searcher.add_font_data(Buffer::from_static(include_bytes!(
            "../../assets/fonts/NewCMMath-Regular.otf"
        )));
        searcher.search_browser().await?;

        Ok(Self {
            // library: Prehashed::new(typst_library::build()),
            book: Prehashed::new(searcher.book),
            fonts: searcher.fonts,
            main: SourceId::detached(),
        })
    }

    fn font(&self, id: usize) -> Option<Font> {
        let slot = &self.fonts[id];
        slot.font
            .get_or_init(|| {
                let font = Font::new(slot.buffer.clone(), slot.index);
                font
            })
            .clone()
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
    fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    fn add_font_data(&mut self, buffer: Buffer) {
        for (i, info) in FontInfo::iter(buffer.as_slice()).enumerate() {
            self.book.push(info);
            self.fonts.push(FontSlot {
                buffer: buffer.clone(),
                index: i as u32,
                font: OnceCell::new(),
            })
        }
    }

    async fn search_browser(&mut self) -> Result<(), JsValue> {
        // if let Some(window) = web_sys::window() {
        //     for fontdata in JsFuture::from(window.query_local_fonts()?)
        //         .await?
        //         .dyn_into::<js_sys::Array>()?
        //         .to_vec()
        //     {
        //         let buffer = Buffer::from(
        //             js_sys::Uint8Array::new(
        //                 &JsFuture::from(
        //                     JsFuture::from(fontdata.dyn_into::<FontData>()?.blob())
        //                         .await?
        //                         .dyn_into::<Blob>()?
        //                         .array_buffer(),
        //                 )
        //                 .await?,
        //             )
        //             .to_vec(),
        //         );
        //         self.add_font_data(buffer);
        //     }
        // }
        Ok(())
    }
}
