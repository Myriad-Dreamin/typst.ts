use std::{path::Path, sync::Arc};

use typst::{diag::PackageResult, file::PackageSpec};

pub mod dummy;

#[cfg(feature = "system")]
pub mod system;

pub trait Registry {
    fn reset(&mut self) {}

    fn resolve(&self, spec: &PackageSpec) -> PackageResult<Arc<Path>>;
}

pub trait Notifier {
    fn downloading(&self, _spec: &PackageSpec) {}
}

#[derive(Debug, Default, Clone, Copy, Hash)]
pub struct DummyNotifier;
impl Notifier for DummyNotifier {}
