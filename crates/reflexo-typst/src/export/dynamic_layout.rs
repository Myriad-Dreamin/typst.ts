use std::{path::PathBuf, sync::Arc};

use reflexo_typst2vec::pass::{CommandExecutor, Typst2VecPass};
use reflexo_typst2vec::IntoTypst;
use reflexo_vec2svg::{DynamicLayoutSvgExporter, MultiVecDocument};
use reflexo_world::TaskInputs;
use typst::diag::Warned;
use typst::foundations::IntoValue;
use typst::utils::LazyHash;
use typst::{diag::SourceResult, World};

use crate::typst::prelude::*;
use crate::vector::ir::{LayoutRegion, LayoutRegionNode};
use crate::world::{CompilerFeat, CompilerWorld};
use crate::{
    CompileEnv, CompileSnapshot, Compiler, Exporter, TypstDict, TypstPagedDocument as Document,
};

pub type LayoutWidths = EcoVec<typst::layout::Abs>;

pub type PostProcessLayoutFn = Arc<
    dyn Fn(&mut Typst2VecPass, Arc<Document>, LayoutRegionNode) -> LayoutRegionNode + Send + Sync,
>;

pub type PostProcessLayoutsFn =
    Arc<dyn Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion> + Send + Sync>;

// todo: derive clone may slow?
pub struct DynamicLayoutCompiler<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> {
    pub compiler: C,
    // todo: abstract this
    output: PathBuf,
    pub extension: String,

    pub layout_widths: LayoutWidths,

    pub command_executor: Arc<dyn CommandExecutor + Send + Sync>,

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

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>> + Clone> Clone
    for DynamicLayoutCompiler<F, C>
{
    fn clone(&self) -> Self {
        Self {
            compiler: self.compiler.clone(),
            output: self.output.clone(),
            extension: self.extension.clone(),
            layout_widths: self.layout_widths.clone(),
            command_executor: self.command_executor.clone(),
            post_process_layout: self.post_process_layout.clone(),
            post_process_layouts: self.post_process_layouts.clone(),
            target: self.target.clone(),
        }
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> DynamicLayoutCompiler<F, C> {
    pub fn new(compiler: C, output: PathBuf) -> Self {
        Self {
            compiler,
            output,
            extension: "multi.sir.in".to_owned(),
            layout_widths: LayoutWidths::from_iter(
                (0..40).map(|i| {
                    typst::layout::Abs::pt(750.0) - typst::layout::Abs::pt(i as f64 * 10.0)
                }),
            ),
            command_executor: Arc::new(()),
            post_process_layout: None,
            post_process_layouts: None,
            target: "web".to_owned(),
        }
    }

    pub fn inner(&self) -> &C {
        &self.compiler
    }

    pub fn inner_mut(&mut self) -> &mut C {
        &mut self.compiler
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
        command_sanitizer: Arc<dyn CommandExecutor + Send + Sync>,
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
        self.post_process_layout = Some(Arc::new(post_process_layout));
    }

    /// Experimental
    pub fn set_post_process_layouts(
        &mut self,
        post_process_layouts: impl Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion>
            + Send
            + Sync
            + 'static,
    ) {
        self.post_process_layouts = Some(Arc::new(post_process_layouts));
    }

    pub fn module_dest_path(&self) -> PathBuf {
        self.output.with_extension(&self.extension)
    }
}

// F: CompilerFeat, CompilerWorld<F>
impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> DynamicLayoutCompiler<F, C> {
    /// Export a typst document using `reflexo_typst::DocumentExporter`.
    pub fn do_export(
        &mut self,
        world: &CompilerWorld<F>,
        env: &mut CompileEnv,
    ) -> SourceResult<(Warned<Arc<Document>>, MultiVecDocument)> {
        let mut svg_exporter = DynamicLayoutSvgExporter::default();
        svg_exporter.typst2vec.command_executor = self.command_executor.clone();
        self.do_export_with(world, env, svg_exporter)
    }

    /// Export a typst document using `reflexo_typst::DocumentExporter`.
    pub fn do_export_with(
        &mut self,
        world: &CompilerWorld<F>,
        env: &mut CompileEnv,
        mut svg_exporter: reflexo_vec2svg::DynamicLayoutSvgExporter,
    ) -> SourceResult<(Warned<Arc<Document>>, MultiVecDocument)> {
        // self.export(doc.clone())?;
        // checkout the entry file

        let mut std_doc = None;

        // for each 10pt we rerender once
        let instant_begin = reflexo::time::Instant::now();
        for (i, current_width) in self.layout_widths.clone().into_iter().enumerate() {
            let instant = reflexo::time::Instant::now();
            // replace layout

            let world = world.task(TaskInputs {
                inputs: Some({
                    let mut dict = TypstDict::new();
                    dict.insert("x-page-width".into(), current_width.into_value());
                    dict.insert("x-target".into(), self.target.clone().into_value());

                    Arc::new(LazyHash::new(dict))
                }),
                ..Default::default()
            });

            log::trace!(
                "rerendering {i} at {:?}, width={current_width:?} target={}",
                instant - instant_begin,
                self.target,
            );

            // compile and export document
            let output = self.compiler.compile(&world, env)?;
            let mut layout = svg_exporter.render(&output.output);

            if let Some(post_process_layout) = &self.post_process_layout {
                layout =
                    post_process_layout(&mut svg_exporter.typst2vec, output.output.clone(), layout);
            }
            svg_exporter
                .layouts
                .push((current_width.into_typst(), layout));

            log::trace!("rerendered {i} at {:?}", instant - instant_begin);

            std_doc = Some(output);
        }

        // post process
        let mut layouts = vec![LayoutRegion::new_by_scalar(
            "width".into(),
            svg_exporter.layouts,
        )];
        if let Some(post_process_layouts) = &self.post_process_layouts {
            layouts = post_process_layouts(&mut svg_exporter.typst2vec, layouts);
        }

        // finalize
        let module = svg_exporter.typst2vec.finalize();
        let doc = MultiVecDocument { module, layouts };

        let instant = reflexo::time::Instant::now();
        log::trace!("multiple layouts finished at {:?}", instant - instant_begin);

        Ok((std_doc.unwrap(), doc))
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>> + Clone> Exporter<CompileSnapshot<F>>
    for DynamicLayoutCompiler<F, C>
{
    fn export(&self, _world: &dyn World, output: Arc<CompileSnapshot<F>>) -> SourceResult<()> {
        self.clone()
            .compile(&output.world, &mut output.env.clone())
            .map(|_| ())
    }
}

impl<F: CompilerFeat, C: Compiler<W = CompilerWorld<F>>> Compiler for DynamicLayoutCompiler<F, C> {
    type W = CompilerWorld<F>;

    fn reset(&mut self) -> SourceResult<()> {
        Ok(())
    }

    fn compile(
        &mut self,
        world: &Self::W,
        env: &mut CompileEnv,
    ) -> SourceResult<Warned<Arc<Document>>> {
        let (res, doc) = self.do_export(world, env)?;
        std::fs::write(self.module_dest_path(), doc.to_bytes()).unwrap();
        Ok(res)
    }
}
