use std::path::Path;

use typst::doc::Document;
use typst_ts_compiler::{
    service::{
        watch_dir, CompileDriver, CompileExporter, Compiler, DiagObserver, DynamicLayoutCompiler,
        WrappedCompiler,
    },
    TypstSystemWorld,
};
use typst_ts_core::{config::CompileOpts, exporter_builtins::GroupExporter, path::PathClean};

use crate::{
    font::EMBEDDED_FONT,
    tracing::TraceGuard,
    utils::{self, UnwrapOrExit},
    CompileArgs, CompileOnceArgs,
};

pub fn create_driver(args: CompileOnceArgs) -> CompileDriver {
    let workspace_dir = Path::new(args.workspace.as_str()).clean();
    let entry_file_path = Path::new(args.entry.as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().unwrap_or_exit();
        cwd.join(workspace_dir)
    };

    let entry_file_path = if entry_file_path.is_absolute() {
        entry_file_path
    } else {
        let cwd = std::env::current_dir().unwrap_or_exit();
        cwd.join(entry_file_path)
    };

    if !entry_file_path.starts_with(&workspace_dir) {
        clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            format!(
                "entry file path must be in workspace directory: {workspace_dir}\n",
                workspace_dir = workspace_dir.display()
            ),
        )
        .exit()
    }

    let world = TypstSystemWorld::new(CompileOpts {
        root_dir: workspace_dir.clone(),
        font_paths: args.font_paths.clone(),
        with_embedded_fonts: EMBEDDED_FONT.to_owned(),
        ..CompileOpts::default()
    })
    .unwrap_or_exit();

    CompileDriver {
        world,
        entry_file: entry_file_path.to_owned(),
    }
}

pub fn compile_export(args: CompileArgs, exporter: GroupExporter<Document>) -> ! {
    if args.trace.is_some() && args.watch {
        clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            "cannot use option \"--trace\" and \"--watch\" at the same time\n",
        )
        .exit()
    }

    let driver = create_driver(args.compile.clone());

    let _trace_guard = {
        let guard = args.trace.clone().map(TraceGuard::new);
        let guard = guard.transpose().map_err(|err| {
            clap::Error::raw(
                clap::error::ErrorKind::InvalidValue,
                format!("init trace failed: {err}\n"),
            )
            .exit()
        });
        guard.unwrap()
    };

    // todo: make dynamic layout exporter
    let output_dir = {
        // If output is specified, use it.
        let dir = (!args.compile.output.is_empty()).then(|| Path::new(&args.compile.output));
        // Otherwise, use the parent directory of the entry file.
        let dir = dir.unwrap_or_else(|| {
            driver
                .entry_file
                .parent()
                .expect("entry_file has no parent")
        });
        dir.join(
            driver
                .entry_file
                .file_name()
                .expect("entry_file has no file name"),
        )
    };

    // CompileExporter
    let driver = CompileExporter::new(driver).with_exporter(exporter);
    let mut driver = DynamicLayoutCompiler::new(driver, output_dir).set_enable(args.dynamic_layout);

    if args.watch {
        utils::async_continue(async move {
            watch_dir(&driver.world().root.clone(), move |events| {
                // relevance checking
                if events.is_some()
                    && !events
                        .unwrap()
                        .iter()
                        // todo: inner
                        .any(|event| driver.inner().inner().relevant(event))
                {
                    return;
                }

                // compile
                driver.with_compile_diag::<true, _>(|driver| driver.compile());
                comemo::evict(30);
            })
            .await;
        })
    } else {
        let compiled = driver.with_compile_diag::<false, _>(|driver| driver.compile());
        utils::logical_exit(compiled.is_some());
    }
}
