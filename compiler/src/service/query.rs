use comemo::Track;
use typst::{
    diag::{EcoString, StrResult},
    doc::Document,
    eval::{eval_string, EvalMode, Scope},
    model::{Content, Introspector, LocatableSelector},
    syntax::Span,
    World,
};

// todo: query exporter
/// Retrieve the matches for the selector.
pub fn retrieve(world: &dyn World, selector: &str, document: &Document) -> StrResult<Vec<Content>> {
    let selector = eval_string(
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
    .cast::<LocatableSelector>()?;

    Ok(Introspector::new(&document.pages)
        .query(&selector.0)
        .into_iter()
        .map(|x| x.into_inner())
        .collect::<Vec<_>>())
}
