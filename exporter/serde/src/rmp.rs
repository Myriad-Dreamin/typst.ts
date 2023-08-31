use std::sync::Arc;

use serde::Serialize;
use typst::{diag::SourceResult, World};
use typst_ts_core::Exporter;

use crate::map_err;

#[derive(Debug, Clone, Default)]
pub struct RmpExporter<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Serialize> Exporter<T, Vec<u8>> for RmpExporter<T> {
    fn export(&self, _world: &dyn World, output: Arc<T>) -> SourceResult<Vec<u8>> {
        let rmp_data = rmp_serde::to_vec_named(output.as_ref());
        rmp_data.map_err(map_err)
    }
}
