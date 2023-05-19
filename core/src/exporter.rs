use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub(crate) type DynExporter<Input, Output = ()> = Box<dyn Exporter<Input, Output>>;

pub trait Exporter<Input, Output = ()> {
    /// Export the given input with given world.
    /// the writable world is hiden by trait itself.
    fn export(&self, world: &dyn World, output: Arc<Input>) -> SourceResult<Output>;
}

/// Lambda can automatically implement the Exporter trait.
impl<I, O, F> Exporter<I, O> for F
where
    F: (for<'a, 'b> Fn(&'a (dyn World + 'b), Arc<I>) -> SourceResult<O>) + Sized,
{
    fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<O> {
        self(world, output)
    }
}

/// This function is used to work around the lifetime issue of a closure lambda.
/// See https://github.com/rust-lang/rust/issues/70263
pub fn mark_exporter_lambda<I, O, F>(f: F) -> F
where
    F: (for<'a, 'b> Fn(&'a (dyn World + 'b), Arc<I>) -> SourceResult<O>) + Sized,
{
    f
}

pub mod builtins {
    use std::sync::Arc;

    use super::{utils, DynExporter, Exporter};
    use typst::{diag::SourceResult, World};

    pub struct GroupExporter<Input> {
        exporters: Vec<DynExporter<Input>>,
    }

    impl<I> GroupExporter<I> {
        pub fn new(exporters: Vec<DynExporter<I>>) -> Self {
            Self { exporters }
        }
    }

    impl<I> Exporter<I> for GroupExporter<I> {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let mut errors = Vec::new();

            for f in &self.exporters {
                utils::collect_err(&mut errors, f.export(world, output.clone()))
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(Box::new(errors))
            }
        }
    }

    /// The Exporter<From<&Input>> must be explicitly constructed.
    pub struct FromExporter<Input, AsInput> {
        exporter: GroupExporter<AsInput>,

        from_input: std::marker::PhantomData<Input>,
    }

    impl<I, A> FromExporter<I, A> {
        pub fn new(exporters: Vec<DynExporter<A>>) -> Self {
            Self {
                exporter: GroupExporter { exporters },
                from_input: std::marker::PhantomData,
            }
        }
    }

    impl<I, A> Exporter<I> for FromExporter<I, A>
    where
        A: for<'a> From<&'a I>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let as_output = output.as_ref().into();
            self.exporter.export(world, Arc::new(as_output))
        }
    }
}

pub mod utils {
    use std::error::Error;
    use typst::{
        diag::{SourceError, SourceResult},
        World,
    };

    pub fn collect_err(errors: &mut Vec<SourceError>, res: SourceResult<()>) {
        if let Err(errs) = res {
            let mut errs = *errs;
            errors.append(&mut errs);
        }
    }

    /// Convert the given error to a vector of source errors.
    pub fn map_err<E: Error>(world: &dyn World, e: E) -> Box<Vec<SourceError>> {
        Box::new(vec![SourceError::new(
            typst::syntax::Span::new(world.main().id(), 0),
            e.to_string(),
        )])
    }

    /// Export document to file system
    pub fn write_to_path<C: AsRef<[u8]>>(
        world: &dyn World,
        path: Option<std::path::PathBuf>,
        content: C,
    ) -> SourceResult<()> {
        path.map_or(Ok(()), |path| {
            std::fs::write(path, content).map_err(|e| map_err(world, e))
        })
    }
}
