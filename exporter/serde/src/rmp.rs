use typst::{diag::SourceResult, World};
use typst_ts_core::ArtifactExporter;

use crate::{map_err, serde_exporter, write_to_path};

serde_exporter!(RmpArtifactExporter);

impl ArtifactExporter for RmpArtifactExporter {
    fn export(&self, world: &dyn World, output: &typst_ts_core::Artifact) -> SourceResult<()> {
        let rmp_doc = rmp_serde::to_vec_named(&output).map_err(|e| map_err(world, e))?;
        write_to_path(world, self.path.clone(), rmp_doc)
    }
}
