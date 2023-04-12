pub(crate) mod system_world;
pub use system_world::SystemFontSearcher;
pub use system_world::TypstSystemWorld;

// todo: make compiler work in browser
// #[cfg(feature = "web")]
// pub(crate) mod browser_world;
// #[cfg(feature = "web")]
// pub use browser_world::BrowserFontSearcher;
// #[cfg(feature = "web")]
// pub use browser_world::TypstBrowserWorld;
