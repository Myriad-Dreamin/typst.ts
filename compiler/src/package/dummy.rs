use std::{path::Path, sync::Arc};

use typst::file::PackageSpec;

use super::Registry;

#[derive(Default, Debug)]
pub struct DummyRegistry;

impl Registry for DummyRegistry {
    fn resolve(&self, spec: &PackageSpec) -> typst::diag::PackageResult<Arc<Path>> {
        Err(typst::diag::PackageError::NotFound(spec.clone()))
    }
}
