use std::{path::PathBuf, sync::Arc};

use crate::ShadowApi;
use typst::{
    diag::{SourceDiagnostic, SourceResult},
    World,
};
use typst_ts_core::{
    exporter_builtins::GroupExporter, typst::prelude::*, DynExporter, DynGenericExporter,
    DynPolymorphicExporter, GenericExporter, TakeAs, TypstDocument, TypstFileId,
};

use super::{
    features::{CompileFeature, FeatureSet, WITH_COMPILING_STATUS_FEATURE},
    CompileEnv, CompileMiddleware, Compiler,
};

pub trait WorldExporter {
    fn export(&mut self, output: Arc<typst::doc::Document>) -> SourceResult<()>;
}

pub struct CompileExporter<C: Compiler> {
    pub compiler: C,
    pub exporter: DynExporter<TypstDocument>,
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

impl<C: Compiler> WorldExporter for CompileExporter<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, output: Arc<typst::doc::Document>) -> SourceResult<()> {
        self.exporter.export(self.compiler.world(), output)
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

    fn wrap_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<typst::doc::Document>> {
        let doc = self.inner_mut().compile(env)?;
        self.export(doc.clone())?;

        Ok(doc)
    }
}

#[derive(Clone, Debug)]
pub enum CompileReport {
    Stage(TypstFileId, &'static str, std::time::SystemTime),
    CompileError(TypstFileId, EcoVec<SourceDiagnostic>, std::time::Duration),
    ExportError(TypstFileId, EcoVec<SourceDiagnostic>, std::time::Duration),
    CompileWarning(TypstFileId, EcoVec<SourceDiagnostic>, std::time::Duration),
    CompileSuccess(TypstFileId, EcoVec<SourceDiagnostic>, std::time::Duration),
}

impl CompileReport {
    pub fn compiling_id(&self) -> TypstFileId {
        match self {
            Self::Stage(id, ..)
            | Self::CompileError(id, ..)
            | Self::ExportError(id, ..)
            | Self::CompileWarning(id, ..)
            | Self::CompileSuccess(id, ..) => *id,
        }
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        match self {
            Self::Stage(..) => None,
            Self::CompileError(_, _, dur)
            | Self::ExportError(_, _, dur)
            | Self::CompileWarning(_, _, dur)
            | Self::CompileSuccess(_, _, dur) => Some(*dur),
        }
    }

    pub fn diagnostics(self) -> Option<EcoVec<SourceDiagnostic>> {
        match self {
            Self::Stage(..) => None,
            Self::CompileError(_, diagnostics, ..)
            | Self::ExportError(_, diagnostics, ..)
            | Self::CompileWarning(_, diagnostics, ..)
            | Self::CompileSuccess(_, diagnostics, ..) => Some(diagnostics),
        }
    }
}

pub type ReportExporter = DynExporter<CompileReport>;
pub type FeaturedReportExporter = DynExporter<(Arc<FeatureSet>, CompileReport)>;

#[allow(dead_code)]
pub struct CompileReporter<C: Compiler> {
    pub compiler: C,
    pub reporter: DynGenericExporter<C::World, (Arc<FeatureSet>, CompileReport)>,
}

#[allow(dead_code)]
impl<C: Compiler> CompileReporter<C>
where
    C::World: 'static,
{
    pub fn new(compiler: C) -> Self {
        let x: FeaturedReportExporter = GroupExporter::new(vec![]).into();
        Self {
            compiler,
            reporter: Box::new(DynPolymorphicExporter::<C::World, _, _>::new(x)),
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
        self.reporter = Box::new(DynPolymorphicExporter::<C::World, _, _>::new(reporter));
    }

    /// Wrap driver with a given featured reporter.
    pub fn with_featured_reporter(mut self, reporter: impl Into<FeaturedReportExporter>) -> Self {
        self.set_featured_reporter(reporter);
        self
    }

    /// set an featured reporter.
    pub fn set_featured_reporter(&mut self, reporter: impl Into<FeaturedReportExporter>) {
        self.reporter = Box::new(DynPolymorphicExporter::<C::World, _, _>::new(
            reporter.into(),
        ));
    }

    /// Wrap driver with a given generic reporter.
    pub fn with_generic_reporter(
        mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = C::World> + Send + 'static,
    ) -> Self {
        self.reporter = Box::new(reporter);
        self
    }

    /// set an generic reporter.
    pub fn set_generic_reporter(
        &mut self,
        reporter: impl GenericExporter<(Arc<FeatureSet>, CompileReport), W = C::World> + Send + 'static,
    ) {
        self.reporter = Box::new(reporter);
    }
}

impl<C: Compiler + WorldExporter> WorldExporter for CompileReporter<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, output: Arc<typst::doc::Document>) -> SourceResult<()> {
        self.compiler.export(output)
    }
}

impl<C: Compiler> CompileMiddleware for CompileReporter<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<typst::doc::Document>> {
        let start = instant::SystemTime::now();
        let id = self.main_id();
        if WITH_COMPILING_STATUS_FEATURE.retrieve(&env.features) {
            let rep = CompileReport::Stage(id, "compiling", start);
            let rep = Arc::new((env.features.clone(), rep));
            // we currently ignore export error here
            let _ = self.reporter.export(self.compiler.world(), rep);
        }

        let tracer = env.tracer.take();
        let origin = tracer.is_some();

        env.tracer = Some(tracer.unwrap_or_default());

        let doc = self.inner_mut().compile(env);

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
        let _ = self.reporter.export(self.compiler.world(), rep);

        doc
    }
}

pub type LayoutWidths = Vec<typst::geom::Abs>;

pub struct DynamicLayoutCompiler<C: Compiler + ShadowApi, const ALWAYS_ENABLE: bool = false> {
    pub compiler: C,

    pub enable_dynamic_layout: bool,

    // todo: abstract this
    output: PathBuf,
    pub extension: String,

    pub layout_widths: LayoutWidths,

    /// Specify the target. It's default value is `web`.
    /// You can specify a sub target like `web-dark` to refine the target.
    /// Though we even don't encourage you to do so.
    ///
    /// Before typst allowing passing arguments to the compiler, this is
    /// (probably) the only way to control the typst code's behavior.
    pub target: String,
}

impl<C: Compiler + ShadowApi> DynamicLayoutCompiler<C> {
    pub fn new(compiler: C, output: PathBuf) -> Self {
        Self {
            compiler,
            output,
            enable_dynamic_layout: false,
            extension: "multi.sir.in".to_owned(),
            layout_widths: LayoutWidths::from_iter(
                (0..40)
                    .map(|i| typst::geom::Abs::pt(750.0) - typst::geom::Abs::pt(i as f64 * 10.0)),
            ),
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

    pub fn with_enable(mut self, enable_dynamic_layout: bool) -> Self {
        self.enable_dynamic_layout = enable_dynamic_layout;
        self
    }
}

#[cfg(feature = "dynamic-layout")]
impl<C: Compiler + ShadowApi> WorldExporter for DynamicLayoutCompiler<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    fn export(&mut self, _output: Arc<typst::doc::Document>) -> SourceResult<()> {
        use std::str::FromStr;

        use typst::{
            diag::At,
            syntax::{PackageSpec, Span, VirtualPath},
        };

        use typst_ts_svg_exporter::{flat_ir::serialize_doc, DynamicLayoutSvgExporter};

        let variable_file = TypstFileId::new(
            Some(PackageSpec::from_str("@preview/typst-ts-variables:0.1.0").at(Span::detached())?),
            VirtualPath::new("lib.typ"),
        );

        // self.export(doc.clone())?;
        // checkout the entry file

        let mut svg_exporter = DynamicLayoutSvgExporter::default();

        // for each 10pt we rerender once
        let instant_begin = instant::Instant::now();
        for (i, current_width) in self.layout_widths.clone().into_iter().enumerate() {
            let instant = instant::Instant::now();
            // replace layout

            let variables: String = format!(
                r##"
#let page-width = {:2}pt
#let target = "{}""##,
                current_width.to_pt(),
                self.target,
            );

            log::trace!(
                "rerendering {} at {:?}, width={current_width:?} target={}",
                i,
                instant - instant_begin,
                self.target,
            );

            self.with_shadow_file_by_id(variable_file, variables.as_bytes().into(), |this| {
                // compile and export document
                let output = this.inner_mut().compile(&mut Default::default())?;
                svg_exporter.render(current_width, output);
                log::trace!(
                    "rerendered {} at {:?}, {}",
                    i,
                    instant - instant_begin,
                    svg_exporter.debug_stat()
                );
                Ok(())
            })?;
        }

        let module_output = self.output.with_extension(&self.extension);

        let doc = svg_exporter.finalize();

        std::fs::write(module_output, serialize_doc(doc)).unwrap();

        let instant = instant::Instant::now();
        log::trace!("multiple layouts finished at {:?}", instant - instant_begin);

        Ok(())
    }
}

#[cfg(feature = "dynamic-layout")]
impl<C: Compiler + ShadowApi> CompileMiddleware for DynamicLayoutCompiler<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self, env: &mut CompileEnv) -> SourceResult<Arc<TypstDocument>> {
        if !self.enable_dynamic_layout {
            return self.inner_mut().compile(env);
        }

        let pure_doc = self.inner_mut().compile(env)?;
        self.export(pure_doc.clone())?;

        Ok(pure_doc)
    }
}
