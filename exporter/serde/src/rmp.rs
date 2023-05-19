use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::{Artifact, Exporter};

use crate::{map_err, serde_exporter, write_to_path};

serde_exporter!(RmpArtifactExporter);

impl Exporter<Artifact> for RmpArtifactExporter {
    fn export(&self, world: &dyn World, output: Arc<Artifact>) -> SourceResult<()> {
        let rmp_doc = rmp_serde::to_vec_named(output.as_ref()).map_err(|e| map_err(world, e))?;
        write_to_path(world, self.path.clone(), rmp_doc)
    }
}
