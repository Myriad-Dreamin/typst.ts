use comemo::Track;
use typst::{
    diag::{At, SourceResult},
    engine::{Route, Sink, Traced},
    foundations::Module,
    syntax::Span,
    World,
};

/// Compile a source file into a module.
///
/// - Returns `Ok(document)` if there were no fatal errors.
/// - Returns `Err(errors)` if there were fatal errors.
///
/// Requires a mutable reference to a tracer. Such a tracer can be created with
/// `Tracer::new()`. Independently of whether compilation succeeded, calling
/// `tracer.warnings()` after compilation will return all compiler warnings.
pub fn evaluate(world: &dyn World, sink: &mut Sink) -> SourceResult<Module> {
    let route = Route::default();

    // Call `track` just once to keep comemo's ID stable.
    let sink = sink.track_mut();

    // Try to evaluate the source file into a module.
    let traced = Traced::default();
    let world = world.track();
    let main = world.main();
    let main = world.source(main).at(Span::detached())?;

    typst::eval::eval(world, traced.track(), sink, route.track(), &main)
}
