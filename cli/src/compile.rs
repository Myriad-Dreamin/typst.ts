use std::{path::Path, sync::Arc};

use typst::{diag::SourceResult, doc::Document};
use typst_ts_compiler::{
    service::{watch_dir, CompileDriver},
    TypstSystemWorld,
};
use typst_ts_core::{config::CompileOpts, exporter_builtins::GroupExporter, path::PathClean};

use crate::{
    font::EMBEDDED_FONT,
    tracing::TraceGuard,
    utils::{self, UnwrapOrExit},
    CompileArgs,
};

pub fn compile_export(args: CompileArgs, exporter: GroupExporter<Document>) -> ! {
    if args.trace.is_some() && args.watch {
        clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            "cannot use option \"--trace\" and \"--watch\" at the same time\n",
        )
        .exit()
    }

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

    let compile_driver_root_dir = workspace_dir.clone();
    let compile_driver = || {
        let world = TypstSystemWorld::new(CompileOpts {
            root_dir: compile_driver_root_dir,
            font_paths: args.font_paths.clone(),
            with_embedded_fonts: EMBEDDED_FONT.to_owned(),
            ..CompileOpts::default()
        })
        .unwrap_or_exit();

        CompileDriver {
            world,
            entry_file: entry_file_path.to_owned(),
            exporter,
        }
    };

    #[allow(clippy::type_complexity)]
    let compile_once: Box<dyn Fn(&mut CompileDriver) -> SourceResult<()>> = if args.dynamic_layout {
        Box::new(|driver: &mut CompileDriver| {
            let output_dir = {
                // If output is specified, use it.
                let dir = (!args.output.is_empty()).then(|| Path::new(&args.output));
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
            CompileDriver::once_dynamic(driver, &output_dir)
        })
    } else {
        Box::new(|driver: &mut CompileDriver| {
            let doc = Arc::new(driver.compile()?);
            driver.export(doc)
        })
    };

    if args.watch {
        utils::async_continue(async move {
            let mut driver = compile_driver();
            watch_dir(&workspace_dir, move |events| {
                // relevance checking
                if events.is_some() && !events.unwrap().iter().any(|event| driver.relevant(event)) {
                    return;
                }

                // compile
                driver.with_compile_diag::<true, _>(&compile_once);
                comemo::evict(30);
            })
            .await;
        })
    } else {
        let compiled = compile_driver().with_compile_diag::<false, _>(compile_once);
        utils::logical_exit(compiled.is_some());
    }
}
