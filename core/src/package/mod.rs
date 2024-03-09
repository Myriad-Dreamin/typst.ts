use std::{path::Path, sync::Arc};

pub use typst::diag::PackageError;
pub use typst::syntax::package::PackageSpec;

pub mod dummy;

pub trait Registry {
    fn reset(&mut self) {}

    fn resolve(&self, spec: &PackageSpec) -> Result<Arc<Path>, PackageError>;
}
