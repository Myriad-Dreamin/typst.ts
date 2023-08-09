use comemo::Track;
use serde::Serialize;
use typst::diag::{bail, StrResult};
use typst::eval::{eval_string, EvalMode};
use typst::model::Introspector;
use typst::World;
use typst_library::prelude::*;

use crate::QueryArgs;

/// Retrieve the matches for the selector.
pub fn retrieve(
    world: &dyn World,
    command: &QueryArgs,
    document: &Document,
) -> StrResult<Vec<Content>> {
    let selector = eval_string(
        world.track(),
        &command.selector,
        Span::detached(),
        EvalMode::Code,
        Scope::default(),
    )
    .map_err(|errors| {
        let mut message = EcoString::from("failed to evaluate selector");
        for (i, error) in errors.into_iter().enumerate() {
            message.push_str(if i == 0 { ": " } else { ", " });
            message.push_str(&error.message);
        }
        message
    })?
    .cast::<LocatableSelector>()?;

    Ok(Introspector::new(&document.pages)
        .query(&selector.0)
        .into_iter()
        .map(|x| x.into_inner())
        .collect::<Vec<_>>())
}

/// Format the query result in the output format.
pub fn format(elements: Vec<Content>, command: &QueryArgs) -> StrResult<String> {
    if command.one && elements.len() != 1 {
        bail!("expected exactly one element, found {}", elements.len())
    }

    let mapped: Vec<_> = elements
        .into_iter()
        .filter_map(|c| match &command.field {
            Some(field) => c.field(field),
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
fn serialize(data: &impl Serialize, format: &str) -> StrResult<String> {
    match format {
        "json" => serde_json::to_string_pretty(data).map_err(|e| eco_format!("{e}")),
        _ => bail!("unsupported serialization format: {}", format),
    }
}
