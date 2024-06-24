use std::sync::Arc;

use typst::{diag::SourceResult, World};

pub type DynExporter<Input, Output = ()> = Box<dyn Exporter<Input, Output> + Send + Sync>;

pub trait Transformer<Input, Output = ()> {
    /// Export the given input with given world. the writable world is hiden by
    /// trait itself.
    fn export(&self, world: &dyn World, output: Input) -> SourceResult<Output>;
}

/// Lambda can automatically implement the Transformer trait.
impl<I, O, F> Transformer<I, O> for F
where
    F: (for<'a> Fn(&'a (dyn World + 'a), I) -> SourceResult<O>) + Sized,
{
    fn export(&self, world: &dyn World, output: I) -> SourceResult<O> {
        self(world, output)
    }
}

pub trait Exporter<Input, Output = ()> {
    /// Export the given input with given world. the writable world is hiden by
    /// trait itself.
    fn export(&self, world: &dyn World, output: Arc<Input>) -> SourceResult<Output>;
}

/// Lambda can automatically implement the Exporter trait.
impl<I, O, F> Exporter<I, O> for F
where
    F: (for<'a> Fn(&'a (dyn World + 'a), Arc<I>) -> SourceResult<O>) + Sized,
{
    fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<O> {
        self(world, output)
    }
}

impl<I, O, F> From<F> for DynExporter<I, O>
where
    F: (for<'a> Fn(&'a (dyn World + 'a), Arc<I>) -> SourceResult<O>)
        + Sized
        + Send
        + Sync
        + 'static,
{
    fn from(f: F) -> Self {
        Box::new(f)
    }
}

pub type DynGenericExporter<X, Input, Output = ()> =
    Arc<dyn GenericExporter<Input, Output, W = X> + Send + Sync>;

pub trait GenericTransformer<Input, Output = ()> {
    type W;

    /// Export the given input with given world. the writable world is hiden by
    /// trait itself.
    fn export(&self, world: &Self::W, output: Input) -> SourceResult<Output>;
}

pub trait GenericExporter<Input, Output = ()> {
    type W;

    /// Export the given input with given world. the writable world is hiden by
    /// trait itself.
    fn export(&self, world: &Self::W, output: Arc<Input>) -> SourceResult<Output>;
}

pub enum DynPolymorphicExporter<W, Input, Output> {
    /// It is just applied to exactly the same world type.
    Just(DynGenericExporter<W, Input, Output>),
    /// A dynamic exporter that can be applied to any dyn world.
    Dyn(DynExporter<Input, Output>),
}

impl<X, Input, Output> DynPolymorphicExporter<X, Input, Output> {
    pub fn new(exporter: DynExporter<Input, Output>) -> Self {
        Self::Dyn(exporter)
    }
    pub fn new_dyn(exporter: DynExporter<Input, Output>) -> Self {
        Self::Dyn(exporter)
    }
}

impl<X, Input, Output> GenericExporter<Input, Output> for DynPolymorphicExporter<X, Input, Output>
where
    X: World,
{
    type W = X;

    fn export(&self, world: &Self::W, output: Arc<Input>) -> SourceResult<Output> {
        match self {
            Self::Just(exporter) => exporter.export(world, output),
            Self::Dyn(exporter) => exporter.export(world, output),
        }
    }
}

pub mod builtins {
    use std::{fs::File, sync::Arc};

    use crate::{exporter_utils::map_err, AsOwnedBytes, AsOwnedString, AsWritable, Transformer};

    use super::{utils, DynExporter, Exporter};
    use ecow::EcoVec;
    use typst::{diag::SourceResult, World};

    pub struct GroupExporter<Input> {
        exporters: Vec<DynExporter<Input>>,
    }

    impl<I> GroupExporter<I> {
        pub fn new(exporters: Vec<DynExporter<I>>) -> Self {
            Self { exporters }
        }

        pub fn push_front(&mut self, exporter: DynExporter<I>) {
            self.exporters.insert(0, exporter)
        }

        pub fn push(&mut self, exporter: DynExporter<I>) {
            self.exporters.push(exporter)
        }
    }

    impl<I> Exporter<I> for GroupExporter<I> {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let mut errors = EcoVec::new();

            for f in &self.exporters {
                utils::collect_err(&mut errors, f.export(world, output.clone()))
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }

    impl<I: 'static> From<GroupExporter<I>> for DynExporter<I> {
        fn from(exporter: GroupExporter<I>) -> Self {
            Box::new(exporter)
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

    impl<I: 'static + Send + Sync, A: 'static> From<FromExporter<I, A>> for DynExporter<I>
    where
        A: for<'a> From<&'a I>,
    {
        fn from(exporter: FromExporter<I, A>) -> Self {
            Box::new(exporter)
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
            std::fs::write(&self.path, vec.as_ref()).map_err(map_err)?;
            Ok(())
        }
    }

    impl<I, E> Exporter<I> for FsPathExporter<AsWritable, E>
    where
        E: Transformer<(Arc<I>, File)>,
    {
        fn export(&self, world: &dyn World, output: Arc<I>) -> SourceResult<()> {
            let file = std::fs::File::create(&self.path).map_err(map_err)?;

            self.exporter.export(world, (output, file))?;
            Ok(())
        }
    }

    impl<I, Bytes: 'static + Send + Sync, E: 'static + Send + Sync> From<FsPathExporter<Bytes, E>>
        for DynExporter<I>
    where
        E: Exporter<I, Bytes>,
        Bytes: AsRef<[u8]>,
    {
        fn from(exporter: FsPathExporter<Bytes, E>) -> Self {
            Box::new(exporter)
        }
    }

    impl<I, E: 'static + Send + Sync> From<FsPathExporter<AsWritable, E>> for DynExporter<I>
    where
        E: Transformer<(Arc<I>, File)>,
    {
        fn from(exporter: FsPathExporter<AsWritable, E>) -> Self {
            Box::new(exporter)
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
    use core::fmt::Display;
    use ecow::{eco_vec, EcoVec};
    use typst::diag::{SourceDiagnostic, SourceResult};

    pub fn collect_err(errors: &mut EcoVec<SourceDiagnostic>, res: SourceResult<()>) {
        if let Err(errs) = res {
            errors.extend(errs);
        }
    }

    /// Convert the given error to a vector of source errors.
    // todo: report the component position
    pub fn map_err<E: Display>(e: E) -> EcoVec<SourceDiagnostic> {
        eco_vec![SourceDiagnostic::error(
            typst::syntax::Span::detached(),
            e.to_string(),
        )]
    }
}
