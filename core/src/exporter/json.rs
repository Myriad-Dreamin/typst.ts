use std::sync::Arc;

use serde::Serialize;
use typst::{diag::SourceResult, World};

use crate::exporter_utils::map_err;
use crate::{Exporter, Transformer};

#[derive(Debug, Clone)]
pub struct JsonExporter<T> {
    should_truncate_precision: bool,

    _marker: std::marker::PhantomData<T>,
}

impl<T> JsonExporter<T> {
    pub fn new(should_truncate_precision: bool) -> Self {
        Self {
            should_truncate_precision,
            _marker: Default::default(),
        }
    }
}

impl<T> Default for JsonExporter<T> {
    fn default() -> Self {
        Self::new(false)
    }
}

impl<T: Serialize> Exporter<T, String> for JsonExporter<T> {
    fn export(&self, _world: &dyn World, output: Arc<T>) -> SourceResult<String> {
        let json_doc = {
            if self.should_truncate_precision {
                let value = &serde_json::to_value(output.as_ref()).map_err(map_err)?;
                serde_json::to_string(&truncate_precision(value))
            } else {
                serde_json::to_string(output.as_ref())
            }
        };
        json_doc.map_err(map_err)
    }
}

impl<W, T: Serialize> Transformer<(Arc<T>, W)> for JsonExporter<T>
where
    W: std::io::Write,
{
    fn export(&self, _world: &dyn World, (output, writer): (Arc<T>, W)) -> SourceResult<()> {
        let json_doc = {
            if self.should_truncate_precision {
                let value = &serde_json::to_value(output.as_ref()).map_err(map_err)?;
                serde_json::to_writer(writer, &truncate_precision(value))
            } else {
                serde_json::to_writer(writer, output.as_ref())
            }
        };
        json_doc.map_err(map_err)
    }
}

fn truncate_precision(output: &serde_json::Value) -> serde_json::Value {
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

    walk_json(output)
}
