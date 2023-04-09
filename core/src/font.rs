use typst::font::{Font, FontBook};

pub trait FontResolver {
    fn font_book(&self) -> &FontBook;
    fn get_font(&self, idx: usize) -> Font;
}
