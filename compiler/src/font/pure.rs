use typst::text::{FontBook, FontInfo};
use typst_ts_core::{
    font::{BufferFontLoader, FontResolverImpl},
    Bytes, FontSlot,
};

/// memory font builder.
#[derive(Debug)]
pub struct MemoryFontBuilder {
    pub book: FontBook,
    pub fonts: Vec<FontSlot>,
}

impl Default for MemoryFontBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<MemoryFontBuilder> for FontResolverImpl {
    fn from(searcher: MemoryFontBuilder) -> Self {
        FontResolverImpl::new(
            searcher.book,
            Default::default(),
            searcher.fonts,
            Default::default(),
        )
    }
}

impl MemoryFontBuilder {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        Self {
            book: FontBook::new(),
            fonts: vec![],
        }
    }

    /// Add an in-memory font.
    pub fn add_memory_font(&mut self, data: Bytes) {
        for (index, info) in FontInfo::iter(&data).enumerate() {
            self.book.push(info.clone());
            self.fonts.push(FontSlot::new_boxed(BufferFontLoader {
                buffer: Some(data.clone()),
                index: index as u32,
            }));
        }
    }
}
