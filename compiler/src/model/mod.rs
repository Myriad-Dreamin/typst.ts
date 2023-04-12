#[cfg(feature = "system")]
pub(crate) mod system_world;
#[cfg(feature = "system")]
pub use {system_world::SystemFontSearcher, system_world::TypstSystemWorld};

// todo: make compiler work in browser
// #[cfg(feature = "web")]
// pub(crate) mod browser_world;
// #[cfg(feature = "web")]
// pub use browser_world::BrowserFontSearcher;
// #[cfg(feature = "web")]
// pub use browser_world::TypstBrowserWorld;
