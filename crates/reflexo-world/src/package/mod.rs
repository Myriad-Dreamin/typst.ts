pub use typst_ts_core::package::{PackageError, PackageSpec, Registry};

#[cfg(feature = "browser")]
pub mod browser;

#[cfg(feature = "system")]
pub mod http;

pub trait Notifier {
    fn downloading(&self, _spec: &PackageSpec) {}
}

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct DummyNotifier;
impl Notifier for DummyNotifier {}
