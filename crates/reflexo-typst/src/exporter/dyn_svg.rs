use std::sync::Arc;

use reflexo_typst2vec::pass::{CommandExecutor, Typst2VecPass};
use reflexo_typst2vec::IntoTypst;
use reflexo_vec2svg::{DynamicLayoutSvgExporter, ExportFeature, MultiVecDocument};
use tinymist_task::ExportTask;
use tinymist_world::{ConfigTask, TaskInputs};
use typst::diag::SourceResult;
use typst::foundations::IntoValue;
use typst::utils::LazyHash;

use super::prelude::*;
use crate::typst::prelude::*;
use crate::vector::ir::{LayoutRegion, LayoutRegionNode};
use crate::world::{CompilerFeat, CompilerWorld};
use crate::{TypstDict, TypstPagedDocument as Document};

pub type LayoutWidths = EcoVec<typst::layout::Abs>;

pub type PostProcessLayoutFn = Arc<
    dyn Fn(&mut Typst2VecPass, Arc<Document>, LayoutRegionNode) -> LayoutRegionNode + Send + Sync,
>;

pub type PostProcessLayoutsFn =
    Arc<dyn Fn(&mut Typst2VecPass, Vec<LayoutRegion>) -> Vec<LayoutRegion> + Send + Sync>;

// todo: abstract this
// output: PathBuf,
// pub extension: String,

// #[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
// #[serde(rename_all = "kebab-case")]
#[derive(Clone)]
pub struct ExportDynSvgModuleTask {
    // #[serde(flatten)]
    pub export: ExportTask,
    // todo: abstract this
    // output: PathBuf,
    // pub extension: String,
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

pub struct DynSvgModuleExport<EF>(std::marker::PhantomData<EF>);

impl<EF: ExportFeature, F: CompilerFeat> WorldComputable<F> for DynSvgModuleExport<EF> {
    type Output = Option<MultiVecDocument>;

    fn compute(graph: &Arc<WorldComputeGraph<F>>) -> Result<Self::Output> {
        type Config = ConfigTask<ExportDynSvgModuleTask>;

        let Some(config) = graph.get::<Config>().transpose()? else {
            return Ok(None);
        };

        Ok(Some(config.do_export(&graph.snap.world)?))
    }
}

impl ExportDynSvgModuleTask {
    pub fn new() -> Self {
        Self {
            // output: PathBuf
            export: ExportTask::default(),
            // output,
            // extension: "multi.sir.in".to_owned(),
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

    // pub fn set_output(&mut self, output: PathBuf) {
    //     self.output = output;
    // }

    // pub fn set_extension(&mut self, extension: String) {
    //     self.extension = extension;
    // }

    // pub fn module_dest_path(&self) -> PathBuf {
    //     self.output.with_extension(&self.extension)
    // }

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
}

impl Default for ExportDynSvgModuleTask {
    fn default() -> Self {
        Self::new()
    }
}

// F: CompilerFeat, CompilerWorld<F>
impl ExportDynSvgModuleTask {
    /// Export a typst document using `reflexo_typst::DocumentExporter`.
    pub fn do_export<F: CompilerFeat>(
        &self,
        world: &CompilerWorld<F>,
    ) -> SourceResult<MultiVecDocument> {
        let mut svg_exporter = DynamicLayoutSvgExporter::default();
        svg_exporter.typst2vec.command_executor = self.command_executor.clone();
        self.do_export_with(world, svg_exporter)
    }

    /// Export a typst document using `reflexo_typst::DocumentExporter`.
    pub fn do_export_with<F: CompilerFeat>(
        &self,
        world: &CompilerWorld<F>,
        mut svg_exporter: reflexo_vec2svg::DynamicLayoutSvgExporter,
    ) -> SourceResult<MultiVecDocument> {
        // self.export(doc.clone())?;
        // checkout the entry file

        // for each 10pt we rerender once
        let instant_begin = reflexo::time::Instant::now();
        for (i, current_width) in self.layout_widths.clone().into_iter().enumerate() {
            let instant = reflexo::time::Instant::now(); // replace layout

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
            // todo: collect warnings and errors here.
            let output = Arc::new(typst::compile::<TypstPagedDocument>(&world).output?);
            let mut layout = svg_exporter.render(&output);

            if let Some(post_process_layout) = &self.post_process_layout {
                layout = post_process_layout(&mut svg_exporter.typst2vec, output.clone(), layout);
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

        // finalize
        let module = svg_exporter.typst2vec.finalize();
        let doc = MultiVecDocument { module, layouts };

        let instant = reflexo::time::Instant::now();
        log::trace!("multiple layouts finished at {:?}", instant - instant_begin);

        Ok(doc)
    }
}
