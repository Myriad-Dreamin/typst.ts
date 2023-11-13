use std::{path::PathBuf, sync::Arc};

use crate::ShadowApi;
use typst::diag::SourceResult;
use typst_ts_core::{exporter_builtins::GroupExporter, DynExporter, TypstDocument};

use super::{CompileEnv, CompileMiddleware, Compiler};

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

        use typst_ts_core::TypstFileId;
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
