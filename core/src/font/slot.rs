use core::fmt;

use typst::font::Font;

use crate::{FontLoader, QueryRef};

type FontSlotInner = QueryRef<Option<Font>, (), Box<dyn FontLoader + Send>>;

/// Lazy Font Reference, load as needed.
pub struct FontSlot(FontSlotInner);

impl FontSlot {
    pub fn with_value(f: Option<Font>) -> Self {
        Self(FontSlotInner::with_value(f))
    }

    pub fn new(f: Box<dyn FontLoader + Send>) -> Self {
        Self(FontSlotInner::with_context(f))
    }

    pub fn new_boxed<F: FontLoader + Send + 'static>(f: F) -> Self {
        Self::new(Box::new(f))
    }

    /// Gets the reference to the font load result (possible uninitialized).
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    pub fn get_uninitialized(&self) -> Option<Option<Font>> {
        let query_res = self.0.get_uninitialized().clone();
        query_res.map(|res| unsafe { res.unwrap_unchecked() })
    }

    /// Gets or make the font load result.
    pub fn get_or_init(&self) -> Option<Font> {
        let res = self.0.compute_with_context(|mut c| Ok(c.load()));
        { unsafe { res.unwrap_unchecked() } }.clone()
    }
}

impl fmt::Debug for FontSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("FontSlot")
            .field(&self.get_uninitialized())
            .finish()
    }
}
