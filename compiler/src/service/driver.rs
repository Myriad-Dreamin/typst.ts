use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use crate::{ShadowApi, TypstSystemWorld};
use codespan_reporting::files::Files;
use typst::{
    diag::{At, SourceResult},
    syntax::{PackageSpec, Span},
};
use typst_ts_core::{
    exporter_builtins::GroupExporter, exporter_utils::map_err, path::PathClean, Exporter, TakeAs,
    TypstFileId,
};
use typst_ts_svg_exporter::{flat_ir::serialize_doc, DynamicLayoutSvgExporter};

use super::{watch_dir, Compiler, DiagObserver, WrappedCompiler};

/// CompileDriver is a driver for typst compiler.
/// It is responsible for operating the compiler without leaking implementation
/// details of the compiler.
pub struct CompileDriver {
    /// World that has access to the file system.
    pub world: TypstSystemWorld,
    /// Path to the entry file.
    pub entry_file: PathBuf,
}

impl CompileDriver {
    /// Create a new driver.
    pub fn new(world: TypstSystemWorld) -> Self {
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
    pub fn set_entry_file<'a>(&mut self, entry_file: impl Into<Cow<'a, PathBuf>>) {
        self.entry_file = entry_file.into().into_owned();
    }

    /// Get the file id for a given path.
    /// Note: only works for files in the workspace instead of external
    /// packages.
    pub fn id_for_path(&self, pb: PathBuf) -> TypstFileId {
        let pb = if pb.is_absolute() {
            let pb = pb.clean();
            let root = self.world.root.clean();
            pb.strip_prefix(root).unwrap().to_owned()
        } else {
            pb
        };
        let pb: PathBuf = Path::new("/").join(pb);
        TypstFileId::new(None, &pb)
    }
}

impl Compiler for CompileDriver {
    type World = TypstSystemWorld;

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
        self.world.reset();

        // checkout the entry file
        let main_id = self.main_id();
        self.world.main = main_id;
        // early error cannot use map_err
        self.world
            .resolve(&self.entry_file, main_id)
            .map_err(map_err)?;

        Ok(())
    }

    /// Check whether a file system event is relevant to the world.
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

impl ShadowApi for CompileDriver {
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

pub struct DynamicLayoutCompiler<C: Compiler + ShadowApi, const ALWAYS_ENABLE: bool = false> {
    pub compiler: C,
    // todo: abstract this
    output_dir: PathBuf,
    pub enable_dynamic_layout: bool,
}

impl<C: Compiler + ShadowApi> DynamicLayoutCompiler<C> {
    pub fn new(compiler: C, output_dir: PathBuf) -> Self {
        Self {
            compiler,
            output_dir,
            enable_dynamic_layout: false,
        }
    }

    pub fn set_output(&mut self, output_dir: PathBuf) {
        self.output_dir = output_dir;
    }

    pub fn with_enable(mut self, enable_dynamic_layout: bool) -> Self {
        self.enable_dynamic_layout = enable_dynamic_layout;
        self
    }
}

impl<C: Compiler + ShadowApi> WrappedCompiler for DynamicLayoutCompiler<C> {
    type Compiler = C;

    fn inner(&self) -> &Self::Compiler {
        &self.compiler
    }

    fn inner_mut(&mut self) -> &mut Self::Compiler {
        &mut self.compiler
    }

    fn wrap_compile(&mut self) -> SourceResult<typst::doc::Document> {
        if !self.enable_dynamic_layout {
            return self.inner_mut().compile();
        }

        let variable_file = TypstFileId::new(
            Some(PackageSpec::from_str("@preview/typst-ts-variables:0.1.0").at(Span::detached())?),
            std::path::Path::new("/lib.typ"),
        );

        let pure_doc = Arc::new(self.inner_mut().compile()?);

        // self.export(doc.clone())?;
        // checkout the entry file

        use typst::geom::Abs;

        let mut svg_exporter = DynamicLayoutSvgExporter::default();
        let base_layout = Abs::pt(750.0);

        // for each 10pt we rerender once
        let instant_begin = std::time::Instant::now();
        for i in 0..40 {
            let instant = std::time::Instant::now();
            // replace layout
            let current_width = base_layout - Abs::pt(i as f64 * 10.0);

            let variables: String = format!(
                r##"
#let page-width = {:2}pt
#let target = "web""##,
                current_width.to_pt()
            );
            println!(
                "rerendering {} at {:?}, {variables}",
                i,
                instant - instant_begin
            );

            self.with_shadow_file_by_id(variable_file, &variables, |this| {
                // compile and export document
                let output = Arc::new(this.inner_mut().compile().unwrap());
                svg_exporter.render(current_width, output);
                println!(
                    "rerendered {} at {:?}, {}",
                    i,
                    instant - instant_begin,
                    svg_exporter.debug_stat()
                );
                Ok(())
            })
            .unwrap();
        }

        let module_output = self.output_dir.with_extension("multi.sir.bin");

        let (doc, glyphs) = svg_exporter.finalize();

        std::fs::write(module_output, serialize_doc(doc, glyphs)).unwrap();

        let instant = std::time::Instant::now();
        println!("rerendering finished at {:?}", instant - instant_begin);

        Ok(pure_doc.take())
    }
}

pub struct WatchDriver<C: Compiler> {
    pub compiler: C,
    pub root: PathBuf,
    pub enable_watch: bool,
}

impl<C: Compiler> WatchDriver<C>
where
    C::World: for<'files> Files<'files, FileId = TypstFileId>,
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

        watch_dir(&self.root.clone(), move |events| {
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
