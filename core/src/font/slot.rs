use std::sync::{Arc, Mutex};

use once_cell::sync::OnceCell;
use typst::font::Font;

use crate::FontLoader;

/// Lazy Font Reference, load as needed.
pub struct FontSlot(
    /// reference to the loaded font
    OnceCell<Option<Font>>,
    /// reference to the font loader
    Arc<Mutex<Option<Box<dyn FontLoader>>>>,
);

impl FontSlot {
    pub fn with_value(f: Option<Font>) -> Self {
        Self(OnceCell::with_value(f), Arc::new(Mutex::new(None)))
    }

    pub fn new(f: Box<dyn FontLoader>) -> Self {
        Self(OnceCell::new(), Arc::new(Mutex::new(Some(f))))
    }

    pub fn new_boxed<F: FontLoader + 'static>(f: F) -> Self {
        Self::new(Box::new(f))
    }

    fn load(&self) -> Option<Font> {
        let mut init_fn = self.1.lock().unwrap();
        init_fn
            .take()
            .expect("the initialization fn is already poisoned")
            .load()
    }

    /// Gets the reference to the font load result.
    ///
    /// Returns `None` if the cell is empty, or being initialized. This
    /// method never blocks.
    pub fn get_uninitialized(&self) -> Option<&Option<Font>> {
        self.0.get()
    }

    /// Gets or make the font load result.
    pub fn get_or_init(&self) -> Option<Font> {
        self.0.get_or_init(|| self.load()).clone()
    }
}
