use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::ArtifactExporter;

use crate::{map_err, serde_exporter, write_to_path};

serde_exporter!(JsonArtifactExporter);

impl ArtifactExporter for JsonArtifactExporter {
    fn export(&self, world: &dyn World, output: Arc<typst_ts_core::Artifact>) -> SourceResult<()> {
        let json_doc = serde_json::to_string(output.as_ref()).map_err(|e| map_err(world, e))?;
        write_to_path(world, self.path.clone(), json_doc)
    }
}
