use comemo::Track;
use reflexo::typst::TypstPagedDocument;
use typst::{
    diag::{EcoString, StrResult},
    foundations::{Content, LocatableSelector, Scope},
    syntax::Span,
    World,
};
use typst_eval::{eval_string, EvalMode};

// todo: query exporter
/// Retrieve the matches for the selector.
pub fn retrieve(
    world: &dyn World,
    selector: &str,
    document: &TypstPagedDocument,
) -> StrResult<Vec<Content>> {
    let selector = eval_string(
        &typst::ROUTINES,
        world.track(),
        selector,
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
    .cast::<LocatableSelector>()
    .map_err(|e| EcoString::from(format!("failed to cast: {}", e.message())))?;

    Ok(document
        .introspector
        .query(&selector.0)
        .into_iter()
        .collect::<Vec<_>>())
}
