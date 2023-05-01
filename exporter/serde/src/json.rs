use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::ArtifactExporter;

use crate::{map_err, serde_exporter, write_to_path};

serde_exporter!(JsonArtifactExporter);

impl ArtifactExporter for JsonArtifactExporter {
    fn export(&self, world: &dyn World, output: Arc<typst_ts_core::Artifact>) -> SourceResult<()> {
        let val = serde_json::to_value(output.as_ref()).map_err(|e| map_err(world, e))?;
        fn walk_json(val: &serde_json::Value) -> serde_json::Value {
            match val {
                serde_json::Value::Array(arr) => {
                    serde_json::json!(arr.iter().map(|x| walk_json(x)).collect::<Vec<_>>())
                }
                serde_json::Value::Object(obj) => {
                    serde_json::json!(obj
                        .iter()
                        .map(|(k, v)| (k.clone(), walk_json(v)))
                        .collect::<serde_json::Map<_, _>>())
                }
                serde_json::Value::Number(x) => {
                    // round to 3 digits
                    if x.is_f64() {
                        if let Some(x) = x.as_f64() {
                            serde_json::json!(((x * 1000.) as i64 as f64) / 1000.)
                        } else {
                            unreachable!()
                        }
                    } else {
                        serde_json::json!(x)
                    }
                }
                x => x.clone(),
            }
        }

        let quan_val = walk_json(&val);

        // let json_doc = serde_json::to_string(output.as_ref()).map_err(|e| map_err(world, e))?;
        let json_doc = serde_json::to_string(&quan_val).map_err(|e| map_err(world, e))?;
        write_to_path(world, self.path.clone(), json_doc)
    }
}
