use typst::font::Font;
use typst_ts_core::FontLoader;

#[cfg(feature = "system")]
pub mod system;

pub struct ReadFontLoader<R> {
    pub read: R,
    pub index: u32,
}

impl<R: std::io::Read + Sized> FontLoader for ReadFontLoader<R> {
    fn load(&mut self) -> Option<Font> {
        let mut buf = vec![];
        self.read.read_to_end(&mut buf).ok()?;
        Font::new(buf.into(), self.index)
    }
}
