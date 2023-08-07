use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub(crate) type DynExporter<Input, Output = ()> = Box<dyn Exporter<Input, Output> + Send>;

pub trait Transformer<Input, Output = ()> {
    /// Export the given input with given world.
    /// the writable world is hiden by trait itself.
    fn export(&self, world: &dyn World, output: Input) -> SourceResult<Output>;
}

/// Lambda can automatically implement the Transformer trait.
impl<I, O, F> Transformer<I, O> for F
where
    F: (for<'a, 'b> Fn(&'a (dyn World + 'b), I) -> SourceResult<O>) + Sized,
{
    fn export(&self, world: &dyn World, output: I) -> SourceResult<O> {
        self(world, output)
    }
}

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
/// See <https://github.com/rust-lang/rust/issues/70263>
pub fn mark_transformer_lambda<I, O, F>(f: F) -> F
where
    F: (for<'a, 'b> Fn(&'a (dyn World + 'b), I) -> SourceResult<O>) + Sized,
{
    f
}

pub mod builtins {
    use std::{fs::File, sync::Arc};

    use crate::{exporter_utils::map_err, AsOwnedBytes, AsOwnedString, AsWritable, Transformer};

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

    pub struct FsPathExporter<Writable, E> {
        path: std::path::PathBuf,
        exporter: E,

        as_bytes: std::marker::PhantomData<Writable>,
    }

    impl<Writable, E> FsPathExporter<Writable, E> {
        pub fn new(path: std::path::PathBuf, exporter: E) -> Self {
            Self {
                path,
                exporter,
                as_bytes: std::marker::PhantomData,
            }
        }
    }

    impl<I, Bytes, E> Exporter<I> for FsPathExporter<Bytes, E>
    where
        E: Exporter<I, Bytes>,
        Bytes: AsRef<[u8]>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let vec = self.exporter.export(world, output)?;
            std::fs::write(&self.path, vec.as_ref()).map_err(|e| map_err(world, e))?;
            Ok(())
        }
    }

    impl<I, E> Exporter<I> for FsPathExporter<AsWritable, E>
    where
        E: Transformer<(Arc<I>, File)>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let file = std::fs::File::create(&self.path).map_err(|e| map_err(world, e))?;

            self.exporter.export(world, (output, file))?;
            Ok(())
        }
    }

    pub struct VecExporter<Writable, E> {
        exporter: E,

        as_bytes: std::marker::PhantomData<Writable>,
    }

    impl<Writable, E> VecExporter<Writable, E> {
        pub fn new(exporter: E) -> Self {
            Self {
                exporter,
                as_bytes: std::marker::PhantomData,
            }
        }
    }

    impl<I, E> Exporter<I, Vec<u8>> for VecExporter<AsOwnedBytes, E>
    where
        E: Exporter<I, Vec<u8>>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<Vec<u8>> {
            let vec = self.exporter.export(world, output)?;
            Ok(vec)
        }
    }

    impl<I, E> Exporter<I, Vec<u8>> for VecExporter<AsOwnedString, E>
    where
        E: Exporter<I, String>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<Vec<u8>> {
            let vec = self.exporter.export(world, output)?;
            Ok(vec.into_bytes())
        }
    }

    impl<I, E> Exporter<I, Vec<u8>> for VecExporter<AsWritable, E>
    where
        E: for<'a> Transformer<(Arc<I>, &'a mut std::io::Cursor<Vec<u8>>)>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<Vec<u8>> {
            let mut cursor = std::io::Cursor::new(Vec::new());
            self.exporter.export(world, (output, &mut cursor))?;
            Ok(cursor.into_inner())
        }
    }
}

pub mod utils {
    use crate::TypstFileId;
    use std::error::Error;
    use typst::{
        diag::{SourceDiagnostic, SourceResult},
        World,
    };

    pub fn collect_err(errors: &mut Vec<SourceDiagnostic>, res: SourceResult<()>) {
        if let Err(errs) = res {
            let mut errs = *errs;
            errors.append(&mut errs);
        }
    }

    /// Convert the given error to a vector of source errors.
    // todo: report the component position
    pub fn map_err<E: Error>(world: &dyn World, e: E) -> Box<Vec<SourceDiagnostic>> {
        map_err_with_id(world.main().id(), e)
    }

    /// Convert the given error to a vector of source errors.
    pub fn map_err_with_id<E: Error>(file_id: TypstFileId, e: E) -> Box<Vec<SourceDiagnostic>> {
        // the source location is the start of the file
        const START_LOC: u64 = typst::syntax::Span::FULL.start;

        Box::new(vec![SourceDiagnostic::error(
            typst::syntax::Span::new(file_id, START_LOC),
            e.to_string(),
        )])
    }
}
