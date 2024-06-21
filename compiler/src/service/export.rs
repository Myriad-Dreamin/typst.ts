use std::{path::PathBuf, sync::Arc};

use typst::{diag::SourceResult, World};
use typst_ts_core::{
    exporter_builtins::GroupExporter,
    typst::prelude::*,
    vector::{
        ir::{LayoutRegion, LayoutRegionNode},
        pass::{CommandExecutor, Typst2VecPass},
    },
    DynExporter, DynGenericExporter, DynPolymorphicExporter, GenericExporter, TakeAs,
    TypstDocument,
};

#[cfg(feature = "dynamic-layout")]
use typst_ts_svg_exporter::MultiVecDocument;

use super::{
    features::{CompileFeature, FeatureSet, WITH_COMPILING_STATUS_FEATURE},
    CompileEnv, CompileMiddleware, CompileReport, Compiler, EntryManager,
};

pub trait WorldExporter<W> {
    fn export(&mut self, world: &W, output: Arc<typst::model::Document>) -> SourceResult<()>;
}

pub struct CompileExporter<C: Compiler> {
    pub compiler: C,
    pub exporter: DynExporter<TypstDocument>,
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
    pub fn with_exporter(mut self, exporter: impl Into<DynExporter<TypstDocument>>) -> Self {
        self.set_exporter(exporter);
        self
    }

    /// set an exporter.
    pub fn set_exporter(&mut self, exporter: impl Into<DynExporter<TypstDocument>>) {
        self.exporter = exporter.into();
    }
}

impl<W: World, C: Compiler> WorldExporter<W> for CompileExporter<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, world: &W, output: Arc<typst::model::Document>) -> SourceResult<()> {
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

    fn wrap_compile(
        &mut self,
        world: &C::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<typst::model::Document>> {
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
    fn export(&mut self, world: &W, output: Arc<typst::model::Document>) -> SourceResult<()> {
        self.compiler.export(world, output)
    }
}

impl<W: World, C: Compiler<W = W>> CompileMiddleware for CompileReporter<C, W>
where
    W: EntryManager,
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
    ) -> SourceResult<Arc<typst::model::Document>> {
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
    dyn Fn(&mut Typst2VecPass, Arc<TypstDocument>, LayoutRegionNode) -> LayoutRegionNode
        + Send
        + Sync,
>;

pub type PostProcessLayoutsFn =
    Box<dyn Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion> + Send + Sync>;

pub struct DynamicLayoutCompiler<C: Compiler, const ALWAYS_ENABLE: bool = false> {
    pub compiler: C,

    pub enable_dynamic_layout: bool,

    // todo: abstract this
    output: PathBuf,
    pub extension: String,

    pub layout_widths: LayoutWidths,

    pub command_executor: Box<dyn CommandExecutor + Send + Sync>,

    post_process_layout: Option<PostProcessLayoutFn>,
    post_process_layouts: Option<PostProcessLayoutsFn>,

    /// Specify the target. It's default value is `web`.
    /// You can specify a sub target like `web-dark` to refine the target.
    /// Though we even don't encourage you to do so.
    ///
    /// Before typst allowing passing arguments to the compiler, this is
    /// (probably) the only way to control the typst code's behavior.
    pub target: String,
}

impl<C: Compiler> DynamicLayoutCompiler<C> {
    pub fn new(compiler: C, output: PathBuf) -> Self {
        Self {
            compiler,
            output,
            enable_dynamic_layout: false,
            extension: "multi.sir.in".to_owned(),
            layout_widths: LayoutWidths::from_iter(
                (0..40).map(|i| {
                    typst::layout::Abs::pt(750.0) - typst::layout::Abs::pt(i as f64 * 10.0)
                }),
            ),
            command_executor: Box::new(()),
            post_process_layout: None,
            post_process_layouts: None,
            target: "web".to_owned(),
        }
    }

    pub fn set_output(&mut self, output: PathBuf) {
        self.output = output;
    }

    pub fn set_extension(&mut self, extension: String) {
        self.extension = extension;
    }

    pub fn set_layout_widths(&mut self, layout_widths: LayoutWidths) {
        self.layout_widths = layout_widths;
    }

    pub fn set_target(&mut self, target: String) {
        self.target = target;
    }

    /// Experimental
    pub fn set_command_executor(
        &mut self,
        command_sanitizer: Box<dyn CommandExecutor + Send + Sync>,
    ) {
        self.command_executor = command_sanitizer;
    }

    /// Experimental
    pub fn set_post_process_layout(
        &mut self,
        post_process_layout: impl Fn(&mut Typst2VecPass, Arc<TypstDocument>, LayoutRegionNode) -> LayoutRegionNode
            + Send
            + Sync
            + 'static,
    ) {
        self.post_process_layout = Some(Box::new(post_process_layout));
    }

    /// Experimental
    pub fn set_post_process_layouts(
        &mut self,
        post_process_layouts: impl Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion>
            + Send
            + Sync
            + 'static,
    ) {
        self.post_process_layouts = Some(Box::new(post_process_layouts));
    }

    pub fn with_enable(mut self, enable_dynamic_layout: bool) -> Self {
        self.enable_dynamic_layout = enable_dynamic_layout;
        self
    }

    pub fn module_dest_path(&self) -> PathBuf {
        self.output.with_extension(&self.extension)
    }
}

#[cfg(feature = "dynamic-layout")]
impl<W: World, C: Compiler<W = W>> DynamicLayoutCompiler<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    pub fn do_export(&mut self, world: &W) -> SourceResult<MultiVecDocument> {
        use typst_ts_svg_exporter::DynamicLayoutSvgExporter;

        let mut svg_exporter = DynamicLayoutSvgExporter::default();
        std::mem::swap(
            &mut self.command_executor,
            &mut svg_exporter.typst2vec.command_executor,
        );
        let res = self.do_export_with(world, svg_exporter);

        res.map(|(doc, s)| {
            self.command_executor = s;
            doc
        })
    }
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    pub fn do_export_with(
        &mut self,
        world: &W,
        mut svg_exporter: typst_ts_svg_exporter::DynamicLayoutSvgExporter,
    ) -> SourceResult<(MultiVecDocument, Box<dyn CommandExecutor + Send + Sync>)> {
        use typst::foundations::IntoValue;
        use typst_ts_core::{IntoTypst, TypstDict};

        // self.export(doc.clone())?;
        // checkout the entry file

        // for each 10pt we rerender once
        let instant_begin = instant::Instant::now();
        for (i, current_width) in self.layout_widths.clone().into_iter().enumerate() {
            let instant = instant::Instant::now();
            // replace layout

            let mut e = CompileEnv {
                args: Some({
                    let mut dict = TypstDict::new();
                    dict.insert("x-page-width".into(), current_width.into_value());
                    dict.insert("x-target".into(), self.target.clone().into_value());

                    Arc::new(Prehashed::new(dict))
                }),
                ..Default::default()
            };

            log::trace!(
                "rerendering {i} at {:?}, width={current_width:?} target={}",
                instant - instant_begin,
                self.target,
            );

            // compile and export document
            let output = self.inner_mut().compile(world, &mut e)?;
            let mut layout = svg_exporter.render(&output);

            if let Some(post_process_layout) = &self.post_process_layout {
                layout = post_process_layout(&mut svg_exporter.typst2vec, output, layout);
            }
            svg_exporter
                .layouts
                .push((current_width.into_typst(), layout));

            log::trace!("rerendered {i} at {:?}", instant - instant_begin);
        }

        // post process
        let mut layouts = vec![LayoutRegion::new_by_scalar(
            "width".into(),
            svg_exporter.layouts,
        )];
        if let Some(post_process_layouts) = &self.post_process_layouts {
            layouts = post_process_layouts(&mut svg_exporter.typst2vec, layouts);
        }

        let sanitizer =
            std::mem::replace(&mut svg_exporter.typst2vec.command_executor, Box::new(()));

        // finalize
        let module = svg_exporter.typst2vec.finalize();
        let doc = MultiVecDocument { module, layouts };

        let instant = instant::Instant::now();
        log::trace!("multiple layouts finished at {:?}", instant - instant_begin);

        Ok((doc, sanitizer))
    }
}

#[cfg(feature = "dynamic-layout")]
impl<W: World, C: Compiler<W = W>> WorldExporter<W> for DynamicLayoutCompiler<C> {
    fn export(&mut self, world: &W, _output: Arc<typst::model::Document>) -> SourceResult<()> {
        let doc = self.do_export(world)?;
        std::fs::write(self.module_dest_path(), doc.to_bytes()).unwrap();
        Ok(())
    }
}

#[cfg(feature = "dynamic-layout")]
impl<W: World, C: Compiler<W = W>> CompileMiddleware for DynamicLayoutCompiler<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(
        &mut self,
        world: &W,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<TypstDocument>> {
        if !self.enable_dynamic_layout {
            return self.inner_mut().compile(world, env);
        }

        let pure_doc = self.inner_mut().compile(world, env)?;
        self.export(world, pure_doc.clone())?;

        Ok(pure_doc)
    }
}
