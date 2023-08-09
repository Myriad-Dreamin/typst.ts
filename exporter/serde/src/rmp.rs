use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::{Artifact, Exporter};

use crate::map_err;

#[derive(Debug, Clone, Default)]
pub struct RmpArtifactExporter;

impl Exporter<Artifact, Vec<u8>> for RmpArtifactExporter {
    fn export(&self, _world: &dyn World, output: Arc<Artifact>) -> SourceResult<Vec<u8>> {
        let rmp_data = rmp_serde::to_vec_named(output.as_ref());
        rmp_data.map_err(map_err)
    }
}
