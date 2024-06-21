use std::{path::PathBuf, sync::Arc};

use typst::diag::SourceResult;
use typst::foundations::IntoValue;
use typst_ts_core::{
    typst::prelude::*,
    vector::{
        ir::{LayoutRegion, LayoutRegionNode},
        pass::{CommandExecutor, Typst2VecPass},
    },
    IntoTypst, TypstDict, TypstDocument as Document,
};
use typst_ts_svg_exporter::{DynamicLayoutSvgExporter, MultiVecDocument};

use crate::service::{CompileEnv, CompileMiddleware, Compiler};
use crate::world::{CompilerFeat, CompilerWorld};

use super::WorldExporter;

pub type LayoutWidths = Vec<typst::layout::Abs>;

pub type PostProcessLayoutFn = Box<
    dyn Fn(&mut Typst2VecPass, Arc<Document>, LayoutRegionNode) -> LayoutRegionNode + Send + Sync,
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
        post_process_layout: impl Fn(&mut Typst2VecPass, Arc<Document>, LayoutRegionNode) -> LayoutRegionNode
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

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> DynamicLayoutCompiler<C> {
    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    pub fn do_export(&mut self, world: &CompilerWorld<F>) -> SourceResult<MultiVecDocument> {
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
        world: &CompilerWorld<F>,
        mut svg_exporter: typst_ts_svg_exporter::DynamicLayoutSvgExporter,
    ) -> SourceResult<(MultiVecDocument, Box<dyn CommandExecutor + Send + Sync>)> {
        // self.export(doc.clone())?;
        // checkout the entry file

        // for each 10pt we rerender once
        let instant_begin = instant::Instant::now();
        for (i, current_width) in self.layout_widths.clone().into_iter().enumerate() {
            let instant = instant::Instant::now();
            // replace layout

            // todo: generalize me
            let world = world.task(Some({
                let mut dict = TypstDict::new();
                dict.insert("x-page-width".into(), current_width.into_value());
                dict.insert("x-target".into(), self.target.clone().into_value());

                Arc::new(Prehashed::new(dict))
            }));

            log::trace!(
                "rerendering {i} at {:?}, width={current_width:?} target={}",
                instant - instant_begin,
                self.target,
            );

            // compile and export document
            let output = self
                .inner_mut()
                .compile(&world, &mut CompileEnv::default())?;
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

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> WorldExporter<CompilerWorld<F>>
    for DynamicLayoutCompiler<C>
{
    fn export(&mut self, world: &CompilerWorld<F>, _output: Arc<Document>) -> SourceResult<()> {
        let doc = self.do_export(world)?;
        std::fs::write(self.module_dest_path(), doc.to_bytes()).unwrap();
        Ok(())
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> CompileMiddleware
    for DynamicLayoutCompiler<C>
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
        world: &CompilerWorld<F>,
        env: &mut CompileEnv,
    ) -> SourceResult<Arc<Document>> {
        if !self.enable_dynamic_layout {
            return self.inner_mut().compile(world, env);
        }

        let pure_doc = self.inner_mut().compile(world, env)?;
        self.export(world, pure_doc.clone())?;

        Ok(pure_doc)
    }
}
