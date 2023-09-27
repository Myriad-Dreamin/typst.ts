pub use typst_ts_core::package::{PackageError, PackageSpec, Registry};

#[cfg(feature = "browser-compile")]
pub mod browser;

#[cfg(feature = "system-compile")]
pub mod http;

pub trait Notifier {
    fn downloading(&self, _spec: &PackageSpec) {}
}

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct DummyNotifier;
impl Notifier for DummyNotifier {}
