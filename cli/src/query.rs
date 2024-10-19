use serde::Serialize;
use typst::{
    diag::{bail, eco_format, StrResult},
    foundations::{Content, IntoValue},
};

use crate::QueryArgs;

/// Format the query result in the output format.
pub fn format(elements: Vec<Content>, command: &QueryArgs) -> StrResult<String> {
    if command.one && elements.len() != 1 {
        bail!("expected exactly one element, found {}", elements.len())
    }

    let mapped: Vec<_> = elements
        .into_iter()
        .filter_map(|c| match &command.field {
            Some(field) => c.get_by_name(field).ok(),
            _ => Some(c.into_value()),
        })
        .collect();

    if command.one {
        serialize(&mapped[0], "json")
    } else {
        serialize(&mapped, "json")
    }
}

/// Serialize data to the output format.
pub fn serialize(data: &impl Serialize, format: &str) -> StrResult<String> {
    match format {
        "json" => serde_json::to_string_pretty(data).map_err(|e| eco_format!("{e}")),
        _ => bail!("unsupported serialization format: {}", format),
    }
}
