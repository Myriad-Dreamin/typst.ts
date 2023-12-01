use comemo::{Track, TrackedMut};
use typst::{diag::SourceResult, engine::Route, eval::Tracer, foundations::Module, World};

/// Compile a source file into a module.
///
/// - Returns `Ok(document)` if there were no fatal errors.
/// - Returns `Err(errors)` if there were fatal errors.
///
/// Requires a mutable reference to a tracer. Such a tracer can be created with
/// `Tracer::new()`. Independently of whether compilation succeeded, calling
/// `tracer.warnings()` after compilation will return all compiler warnings.
pub fn evaluate(world: &dyn World, tracer: &mut Tracer) -> SourceResult<Module> {
    let route = Route::default();

    // Call `track` just once to keep comemo's ID stable.
    let world = world.track();
    let mut tracer = tracer.track_mut();

    // Try to evaluate the source file into a module.
    typst::eval::eval(
        world,
        route.track(),
        TrackedMut::reborrow_mut(&mut tracer),
        &world.main(),
    )
}
