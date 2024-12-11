use std::sync::Arc;

use reflexo_typst2vec::pass::Typst2VecPass;
use reflexo_world::{CompilerFeat, CompilerWorld};
use typst::{
    diag::{SourceResult, Warned},
    World,
};

use crate::{
    vector::ir::{LayoutRegion, LayoutRegionNode},
    CompiledArtifact,
};

#[cfg(feature = "dynamic-layout")]
mod dynamic_layout;
#[cfg(feature = "dynamic-layout")]
pub use dynamic_layout::*;

use super::{
    features::{CompileFeature, FeatureSet, WITH_COMPILING_STATUS_FEATURE},
    CompileEnv, CompileMiddleware, CompileReport, Compiler,
};
use crate::{
    exporter_builtins::GroupExporter, typst::prelude::*, DynExporter, DynGenericExporter,
    DynPolymorphicExporter, Exporter, GenericExporter, TakeAs, TypstPagedDocument as Document,
};
use crate::{CompileSnapshot, EntryReader};

pub struct CompileStarter<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> {
    pub compiler: C,
    f: std::marker::PhantomData<F>,
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> CompileStarter<F, C> {
    pub fn new(compiler: C) -> Self {
        Self {
            compiler,
            f: std::marker::PhantomData,
        }
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>> + Clone> Exporter<CompiledArtifact<F>>
    for CompileStarter<F, C>
{
    fn export(&self, _world: &dyn World, output: Arc<CompiledArtifact<F>>) -> SourceResult<()> {
        self.compiler
            .clone()
            .compile(&output.world, &mut output.env.clone())
            .map(|_| ())
    }
}

#[derive(Clone)]
pub struct CompileExporter<C: Compiler> {
    pub compiler: C,
    pub exporter: Arc<DynExporter<Document>>,
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
            exporter: Arc::new(GroupExporter::new(vec![]).into()),
        }
    }

    /// Wrap driver with a given exporter.
    pub fn with_exporter(mut self, exporter: impl Into<DynExporter<Document>>) -> Self {
        self.set_exporter(exporter);
        self
    }

    /// set an exporter.
    pub fn set_exporter(&mut self, exporter: impl Into<DynExporter<Document>>) {
        self.exporter = Arc::new(exporter.into());
    }
}

impl<F: CompilerFeat + 'static, C: Compiler> Exporter<CompileSnapshot<F>> for CompileExporter<C> {
    /// Export a typst document using `reflexo_typst::DocumentExporter`.
    fn export(&self, world: &dyn World, output: Arc<CompileSnapshot<F>>) -> SourceResult<()> {
        if let Ok(doc) = output.compile().doc {
            self.exporter.export(world, doc)?;
        }

        Ok(())
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

    fn wrap_compile(
        &mut self,
        world: &C::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Warned<Arc<Document>>> {
        let doc = self.inner_mut().compile(world, env)?;
        self.exporter.export(world, doc.output.clone())?;

        Ok(doc)
    }
}

pub type ReportExporter = DynExporter<CompileReport>;
pub type FeaturedReportExporter = DynExporter<(Arc<FeatureSet>, CompileReport)>;

pub struct CompileReporter<C: Compiler, W: World> {
    pub compiler: C,
    pub reporter: DynGenericExporter<W, (Arc<FeatureSet>, CompileReport)>,
}

impl<C: Compiler + Clone, W: World> Clone for CompileReporter<C, W> {
    fn clone(&self) -> Self {
        Self {
            compiler: self.compiler.clone(),
            reporter: self.reporter.clone(),
        }
    }
}

impl<C: Compiler, W: World + 'static> CompileReporter<C, W> {
    pub fn new(compiler: C) -> Self {
        let x: FeaturedReportExporter = GroupExporter::new(vec![]).into();
        Self {
            compiler,
            reporter: Arc::new(DynPolymorphicExporter::<W, _, _>::new(x)),
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
        self.reporter = Arc::new(DynPolymorphicExporter::<W, _, _>::new(reporter));
    }

    /// Wrap driver with a given featured reporter.
    pub fn with_featured_reporter(mut self, reporter: impl Into<FeaturedReportExporter>) -> Self {
        self.set_featured_reporter(reporter);
        self
    }

    /// set an featured reporter.
    pub fn set_featured_reporter(&mut self, reporter: impl Into<FeaturedReportExporter>) {
        self.reporter = Arc::new(DynPolymorphicExporter::<W, _, _>::new(reporter.into()));
    }

    /// Wrap driver with a given generic reporter.
    pub fn with_generic_reporter(
        mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = W> + Send + Sync + 'static,
    ) -> Self {
        self.reporter = Arc::new(reporter);
        self
    }

    /// set an generic reporter.
    pub fn set_generic_reporter(
        &mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = W> + Send + Sync + 'static,
    ) {
        self.reporter = Arc::new(reporter);
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

    fn wrap_compile(
        &mut self,
        world: &C::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Warned<Arc<Document>>> {
        let start = reflexo::time::now();
        // todo unwrap main id
        let id = world.main_id().unwrap();
        if WITH_COMPILING_STATUS_FEATURE.retrieve(&env.features) {
            let rep = CompileReport::Stage(id, "compiling", start);
            let rep = Arc::new((env.features.clone(), rep));
            // we currently ignore export error here
            let _ = self.reporter.export(world, rep);
        }

        let doc = self.inner_mut().compile(world, env);

        let elapsed = start.elapsed().unwrap_or_default();

        let rep;

        let doc = match doc {
            Ok(doc) => {
                rep = CompileReport::CompileSuccess(id, doc.warnings.clone(), elapsed);

                Ok(doc)
            }
            Err(err) => {
                rep = CompileReport::CompileError(id, err, elapsed);
                Err(eco_vec![])
            }
        };

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
