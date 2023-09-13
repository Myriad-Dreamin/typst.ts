use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::ShadowApi;
use typst::{diag::SourceResult, syntax::VirtualPath, World};
use typst_ts_core::{
    exporter_builtins::GroupExporter, path::PathClean, Exporter, TakeAs, TypstFileId,
};

use super::{Compiler, WorkspaceProvider, WrappedCompiler};

/// CompileDriverImpl is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriverImpl<W: World> {
    /// World that has access to the file system.
    pub world: W,
    /// Path to the entry file.
    pub entry_file: PathBuf,
}

impl<W: World> CompileDriverImpl<W> {
    /// Create a new driver.
    pub fn new(world: W) -> Self {
        Self {
            world,
            entry_file: PathBuf::default(),
        }
    }

    /// Wrap driver with a given entry file.
    pub fn with_entry_file(mut self, entry_file: PathBuf) -> Self {
        self.entry_file = entry_file;
        self
    }

    /// set an entry file.
    pub fn set_entry_file(&mut self, entry_file: PathBuf) {
        self.entry_file = entry_file;
    }
}

impl<W: World + WorkspaceProvider> CompileDriverImpl<W> {
    /// Get the file id for a given path.
    /// Note: only works for files in the workspace instead of external
    /// packages.
    pub fn id_for_path(&self, pb: PathBuf) -> TypstFileId {
        let pb = if pb.is_absolute() {
            let pb = pb.clean();
            let root = self.world.workspace_root().clean();
            pb.strip_prefix(root).unwrap().to_owned()
        } else {
            pb
        };
        TypstFileId::new(None, VirtualPath::new(pb))
    }
}

impl<W: World + WorkspaceProvider> Compiler for CompileDriverImpl<W> {
    type World = W;

    fn world(&self) -> &Self::World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut Self::World {
        &mut self.world
    }

    fn main_id(&self) -> TypstFileId {
        self.id_for_path(self.entry_file.clone())
    }

    /// reset the compilation state
    fn reset(&mut self) -> SourceResult<()> {
        // reset the world caches
        self.world.reset()?;
        // checkout the entry file
        self.world.set_main_id(self.main_id());

        Ok(())
    }

    /// Check whether a file system event is relevant to the world.
    // todo: remove cfg feature here
    #[cfg(feature = "system-watch")]
    fn relevant(&self, event: &notify::Event) -> bool {
        // todo: remove this check
        if event
            .paths
            .iter()
            .any(|p| p.to_string_lossy().contains(".artifact."))
        {
            return false;
        }

        self._relevant(event).unwrap_or(true)
    }
}

impl<W: World + ShadowApi> ShadowApi for CompileDriverImpl<W> {
    fn _shadow_map_id(&self, file_id: TypstFileId) -> typst::diag::FileResult<PathBuf> {
        self.world._shadow_map_id(file_id)
    }

    fn reset_shadow(&mut self) {
        self.world.reset_shadow()
    }

    fn map_shadow(&self, path: &Path, content: &str) -> typst::diag::FileResult<()> {
        self.world.map_shadow(path, content)
    }

    fn unmap_shadow(&self, path: &Path) -> typst::diag::FileResult<()> {
        self.world.unmap_shadow(path)
    }
}

pub struct CompileExporter<C: Compiler> {
    pub compiler: C,
    pub exporter: GroupExporter<typst::doc::Document>,
}

impl<C: Compiler> CompileExporter<C> {
    pub fn new(compiler: C) -> Self {
        Self {
            compiler,
            exporter: GroupExporter::new(vec![]),
        }
    }

    /// Wrap driver with a given exporter.
    pub fn with_exporter(mut self, exporter: GroupExporter<typst::doc::Document>) -> Self {
        self.exporter = exporter;
        self
    }

    /// set an exporter.
    pub fn set_exporter(&mut self, exporter: GroupExporter<typst::doc::Document>) {
        self.exporter = exporter;
    }

    /// Export a typst document using `typst_ts_core::DocumentExporter`.
    pub fn export(&self, output: Arc<typst::doc::Document>) -> SourceResult<()> {
        self.exporter.export(self.compiler.world(), output)
    }
}

impl<C: Compiler> WrappedCompiler for CompileExporter<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self) -> SourceResult<typst::doc::Document> {
        let doc = Arc::new(self.inner_mut().compile()?);
        self.export(doc.clone())?;

        // Note: when doc is not retained by the exporters, no clone happens,
        // because of the `Arc` type detecting a single owner at runtime.
        Ok(doc.take())
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
impl<C: Compiler + ShadowApi> WrappedCompiler for DynamicLayoutCompiler<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self) -> SourceResult<typst::doc::Document> {
        use std::str::FromStr;
        use typst::{
            diag::At,
            syntax::{PackageSpec, Span},
        };
        use typst_ts_svg_exporter::{flat_ir::serialize_doc, DynamicLayoutSvgExporter};

        if !self.enable_dynamic_layout {
            return self.inner_mut().compile();
        }

        let variable_file = TypstFileId::new(
            Some(PackageSpec::from_str("@preview/typst-ts-variables:0.1.0").at(Span::detached())?),
            VirtualPath::new("lib.typ"),
        );

        let pure_doc = Arc::new(self.inner_mut().compile()?);

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

            self.with_shadow_file_by_id(variable_file, &variables, |this| {
                // compile and export document
                let output = Arc::new(this.inner_mut().compile()?);
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

        Ok(pure_doc.take())
    }
}

pub struct WatchDriver<C: Compiler> {
    pub compiler: C,
    pub root: PathBuf,
    pub enable_watch: bool,
}

// todo: remove cfg feature here
#[cfg(feature = "system-watch")]
use super::DiagObserver;
#[cfg(feature = "system-watch")]
impl<C: Compiler> WatchDriver<C>
where
    C::World: for<'files> codespan_reporting::files::Files<'files, FileId = TypstFileId>,
{
    pub fn new(compiler: C, root: PathBuf) -> Self {
        Self {
            compiler,
            root,
            enable_watch: false,
        }
    }

    pub fn with_enable(mut self, enable_watch: bool) -> Self {
        self.enable_watch = enable_watch;
        self
    }

    pub async fn compile(&mut self) -> bool {
        if !self.enable_watch {
            let compiled = self
                .compiler
                .with_compile_diag::<false, _>(|driver| driver.compile());
            return compiled.is_some();
        }

        super::watch_dir(&self.root.clone(), move |events| {
            // relevance checking
            if events.is_some()
                && !events
                    .unwrap()
                    .iter()
                    // todo: inner
                    .any(|event| self.compiler.relevant(event))
            {
                return;
            }

            // compile
            self.compiler
                .with_compile_diag::<true, _>(|driver| driver.compile());
            comemo::evict(30);
        })
        .await;
        true
    }
}

// todo: Print that a package downloading is happening.
// fn print_downloading(_spec: &PackageSpec) -> std::io::Result<()> {
// let mut w = color_stream();
// let styles = term::Styles::default();

// w.set_color(&styles.header_help)?;
// write!(w, "downloading")?;

// w.reset()?;
// writeln!(w, " {spec}")
// Ok(())
// }
