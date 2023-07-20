use std::{path::Path, sync::Arc};

use super::{PackageError, PackageSpec, Registry};

#[derive(Default, Debug)]
pub struct DummyRegistry;

impl Registry for DummyRegistry {
    fn resolve(&self, spec: &PackageSpec) -> Result<Arc<Path>, PackageError> {
        Err(PackageError::NotFound(spec.clone()))
    }
}
