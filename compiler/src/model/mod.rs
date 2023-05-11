pub(crate) mod source_manager;
pub(crate) mod world;

#[cfg(feature = "system")]
pub(crate) mod system;
#[cfg(feature = "system")]
pub use system::{SystemFontSearcher, TypstSystemWorld};

// todo: make compiler work in browser
#[cfg(feature = "web")]
pub(crate) mod browser_world;
#[cfg(feature = "web")]
pub use browser_world::{BrowserFontSearcher, TypstBrowserWorld};
