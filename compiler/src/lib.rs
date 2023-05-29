pub(crate) mod macros;

pub mod font;
pub mod vfs;
pub mod workspace;
pub mod world;

#[cfg(feature = "system")]
pub mod service;

#[cfg(feature = "system")]
pub(crate) mod system;
#[cfg(feature = "system")]
pub use system::TypstSystemWorld;

// todo: make compiler work in browser
#[cfg(feature = "browser-compile")]
pub(crate) mod browser;
#[cfg(feature = "browser-compile")]
pub use browser::TypstBrowserWorld;
