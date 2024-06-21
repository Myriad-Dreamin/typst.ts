use std::sync::Arc;

use typst::{diag::SourceResult, World};
use typst_ts_core::{
    exporter_builtins::GroupExporter,
    typst::prelude::*,
    vector::{
        ir::{LayoutRegion, LayoutRegionNode},
        pass::Typst2VecPass,
    },
    DynExporter, DynGenericExporter, DynPolymorphicExporter, GenericExporter, TakeAs,
    TypstDocument as Document,
};

#[cfg(feature = "dynamic-layout")]
mod dynamic_layout;
#[cfg(feature = "dynamic-layout")]
pub use dynamic_layout::*;

use super::{
    features::{CompileFeature, FeatureSet, WITH_COMPILING_STATUS_FEATURE},
    CompileEnv, CompileMiddleware, CompileReport, Compiler, EntryReader,
};

pub trait WorldExporter<W> {
    fn export(&mut self, world: &W, output: Arc<Document>) -> SourceResult<()>;
}

pub struct CompileExporter<C: Compiler> {
    pub compiler: C,
    pub exporter: DynExporter<Document>,
}

impl<C: Compiler + Default> Default for CompileExporter<C> {
    fn default() -> Self {
        Self::new(C::default())
    }
}

impl<C: Compiler> CompileExporter<C> {
    pub fn new(compiler: C) -> Self {
        Self {
            compiler,
            exporter: GroupExporter::new(vec![]).into(),
        }
    }

    /// Wrap driver with a given exporter.
    pub fn with_exporter(mut self, exporter: impl Into<DynExporter<Document>>) -> Self {
        self.set_exporter(exporter);
        self
    }

    /// set an exporter.
    pub fn set_exporter(&mut self, exporter: impl Into<DynExporter<Document>>) {
        self.exporter = exporter.into();
    }
}

impl<W: World, C: Compiler> WorldExporter<W> for CompileExporter<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, world: &W, output: Arc<Document>) -> SourceResult<()> {
        self.exporter.export(world, output)
    }
}

impl<C: Compiler> CompileMiddleware for CompileExporter<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self, world: &C::W, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        let doc = self.inner_mut().compile(world, env)?;
        self.export(world, doc.clone())?;

        Ok(doc)
    }
}

pub type ReportExporter = DynExporter<CompileReport>;
pub type FeaturedReportExporter = DynExporter<(Arc<FeatureSet>, CompileReport)>;

pub struct CompileReporter<C: Compiler, W: World> {
    pub compiler: C,
    pub reporter: DynGenericExporter<W, (Arc<FeatureSet>, CompileReport)>,
}

impl<C: Compiler, W: World + 'static> CompileReporter<C, W> {
    pub fn new(compiler: C) -> Self {
        let x: FeaturedReportExporter = GroupExporter::new(vec![]).into();
        Self {
            compiler,
            reporter: Box::new(DynPolymorphicExporter::<W, _, _>::new(x)),
        }
    }

    /// Wrap driver with a given reporter.
    pub fn with_reporter(mut self, reporter: impl Into<ReportExporter>) -> Self {
        self.set_reporter(reporter);
        self
    }

    /// set an reporter.
    pub fn set_reporter(&mut self, reporter: impl Into<ReportExporter>) {
        let reporter = reporter.into();
        let reporter: FeaturedReportExporter = Box::new(
            move |world: &dyn World, inp: Arc<(Arc<FeatureSet>, CompileReport)>| {
                // it is believed that no clone will happen here
                reporter.export(world, Arc::new(inp.take().1))
            },
        );
        self.reporter = Box::new(DynPolymorphicExporter::<W, _, _>::new(reporter));
    }

    /// Wrap driver with a given featured reporter.
    pub fn with_featured_reporter(mut self, reporter: impl Into<FeaturedReportExporter>) -> Self {
        self.set_featured_reporter(reporter);
        self
    }

    /// set an featured reporter.
    pub fn set_featured_reporter(&mut self, reporter: impl Into<FeaturedReportExporter>) {
        self.reporter = Box::new(DynPolymorphicExporter::<W, _, _>::new(reporter.into()));
    }

    /// Wrap driver with a given generic reporter.
    pub fn with_generic_reporter(
        mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = W> + Send + 'static,
    ) -> Self {
        self.reporter = Box::new(reporter);
        self
    }

    /// set an generic reporter.
    pub fn set_generic_reporter(
        &mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = W> + Send + 'static,
    ) {
        self.reporter = Box::new(reporter);
    }
}

impl<W: World, C: Compiler + WorldExporter<W>> WorldExporter<W> for CompileReporter<C, W> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, world: &W, output: Arc<Document>) -> SourceResult<()> {
        self.compiler.export(world, output)
    }
}

impl<W: World, C: Compiler<W = W>> CompileMiddleware for CompileReporter<C, W>
where
    W: EntryReader,
{
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self, world: &C::W, env: &mut CompileEnv) -> SourceResult<Arc<Document>> {
        let start = crate::time::now();
        // todo unwrap main id
        let id = world.main_id().unwrap();
        if WITH_COMPILING_STATUS_FEATURE.retrieve(&env.features) {
            let rep = CompileReport::Stage(id, "compiling", start);
            let rep = Arc::new((env.features.clone(), rep));
            // we currently ignore export error here
            let _ = self.reporter.export(world, rep);
        }

        let tracer = env.tracer.take();
        let origin = tracer.is_some();

        env.tracer = Some(tracer.unwrap_or_default());

        let doc = self.inner_mut().compile(world, env);

        let elapsed = start.elapsed().unwrap_or_default();

        let rep;

        let doc = match doc {
            Ok(doc) => {
                let warnings = env.tracer.as_ref().unwrap().clone().warnings();
                if warnings.is_empty() {
                    rep = CompileReport::CompileSuccess(id, warnings, elapsed);
                } else {
                    rep = CompileReport::CompileWarning(id, warnings, elapsed);
                }

                Ok(doc)
            }
            Err(err) => {
                rep = CompileReport::CompileError(id, err, elapsed);
                Err(eco_vec![])
            }
        };

        if !origin {
            env.tracer = None;
        }

        let rep = Arc::new((env.features.clone(), rep));
        // we currently ignore export error here
        let _ = self.reporter.export(world, rep);

        doc
    }
}

pub type LayoutWidths = Vec<typst::layout::Abs>;

pub type PostProcessLayoutFn = Box<
    dyn Fn(&mut Typst2VecPass, Arc<Document>, LayoutRegionNode) -> LayoutRegionNode + Send + Sync,
>;

pub type PostProcessLayoutsFn =
    Box<dyn Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion> + Send + Sync>;
