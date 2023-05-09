use std::io::Read;
use typst::font::Font;
use typst_ts_core::FontLoader;

#[cfg(feature = "system")]
pub mod system;

pub struct ReadFontLoader {
    pub read: Box<dyn std::io::Read>,
    pub index: u32,
}

impl FontLoader for ReadFontLoader {
    fn load(&mut self) -> Option<Font> {
        let mut buf = vec![];
        self.read.read_to_end(&mut buf).ok()?;
        Font::new(buf.into(), self.index)
    }
}
