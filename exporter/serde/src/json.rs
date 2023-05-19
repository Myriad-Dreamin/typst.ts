use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::{Artifact, Exporter};

use crate::map_err;

#[derive(Debug, Clone, Default)]
pub struct JsonArtifactExporter {
    should_truncate_precision: bool,
}

impl JsonArtifactExporter {
    pub fn new(should_truncate_precision: bool) -> Self {
        Self {
            should_truncate_precision,
        }
    }
}

impl Exporter<Artifact, String> for JsonArtifactExporter {
    fn export(&self, world: &dyn World, output: Arc<Artifact>) -> SourceResult<String> {
        let json_doc = {
            if self.should_truncate_precision {
                serde_json::to_string(&self.truncate_precision(world, output)?)
            } else {
                serde_json::to_string(output.as_ref())
            }
        };
        json_doc.map_err(|e| map_err(world, e))
    }
}

impl JsonArtifactExporter {
    fn truncate_precision(
        &self,
        world: &dyn World,
        output: Arc<typst_ts_core::Artifact>,
    ) -> SourceResult<serde_json::Value> {
        fn walk_json(val: &serde_json::Value) -> serde_json::Value {
            match val {
                serde_json::Value::Array(arr) => {
                    serde_json::json!(arr.iter().map(walk_json).collect::<Vec<_>>())
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

        Ok(walk_json(
            &serde_json::to_value(output.as_ref()).map_err(|e| map_err(world, e))?,
        ))
    }
}
