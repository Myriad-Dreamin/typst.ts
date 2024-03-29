use std::borrow::Cow;
use std::io::{self, Read};
use std::path::Path;

use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Dict, IntoValue};
use typst::model::Document;
use typst_ts_compiler::service::EntryManager;
use typst_ts_compiler::ShadowApi;
use typst_ts_compiler::{
    service::{
        features::{FeatureSet, DIAG_FMT_FEATURE},
        CompileActor, CompileDriver, CompileExporter, DynamicLayoutCompiler,
    },
    TypstSystemWorld,
};
use typst_ts_core::config::compiler::{EntryOpts, MEMORY_MAIN_ENTRY};
use typst_ts_core::{config::CompileOpts, exporter_builtins::GroupExporter, path::PathClean};

use crate::font::fonts;
use crate::utils::current_dir;
use crate::{
    tracing::TraceGuard,
    utils::{self, UnwrapOrExit},
    CompileArgs, CompileOnceArgs,
};

pub fn create_driver(args: CompileOnceArgs) -> CompileDriver {
    let workspace_dir = Path::new(args.workspace.as_str()).clean();
    let entry = args.entry;
    let entry_file_path = Path::new(entry.as_str()).clean();

    let workspace_dir = if workspace_dir.is_absolute() {
        workspace_dir
    } else {
        let cwd = std::env::current_dir().unwrap_or_exit();
        cwd.join(workspace_dir)
    };

    let workspace_dir = workspace_dir.clean();

    let is_stdin = entry == "-";
    let entry_file_path = if is_stdin || entry_file_path.is_absolute() {
        entry_file_path
    } else {
        let cwd = std::env::current_dir().unwrap_or_exit();
        cwd.join(entry_file_path)
    };

    let entry_file_path = entry_file_path.clean();

    if !is_stdin && !entry_file_path.starts_with(&workspace_dir) {
        clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            format!(
                "entry file path must be in workspace directory: {workspace_dir}\n",
                workspace_dir = workspace_dir.display()
            ),
        )
        .exit()
    }

    // Convert the input pairs to a dictionary.
    let inputs: Dict = args
        .inputs
        .iter()
        .map(|(k, v)| (k.as_str().into(), v.as_str().into_value()))
        .collect();

    let world = TypstSystemWorld::new(CompileOpts {
        entry: EntryOpts::new_workspace(workspace_dir.clone()),
        inputs,
        font_paths: args.font.paths.clone(),
        with_embedded_fonts: fonts()
            .map(Cow::Borrowed)
            .chain(args.extra_embedded_fonts)
            .collect(),
        ..CompileOpts::default()
    })
    .unwrap_or_exit();

    if is_stdin {
        let mut driver = CompileDriver::new(world);

        let entry = driver.world.entry.select_in_workspace(*MEMORY_MAIN_ENTRY);
        driver.world.mutate_entry(entry).unwrap();

        let src = read_from_stdin()
            .map_err(|err| {
                clap::Error::raw(
                    clap::error::ErrorKind::Io,
                    format!("read from stdin failed: {err}\n"),
                )
                .exit()
            })
            .unwrap();

        driver
            .map_shadow_by_id(*MEMORY_MAIN_ENTRY, Bytes::from(src))
            .map_err(|err| {
                clap::Error::raw(
                    clap::error::ErrorKind::Io,
                    format!("map stdin failed: {err}\n"),
                )
                .exit()
            })
            .unwrap();

        driver
    } else {
        CompileDriver::new(world).with_entry_file(entry_file_path)
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

    let is_stdin = args.compile.entry == "-";
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
        let dir = dir.map(Path::to_owned).unwrap_or_else(|| {
            if is_stdin {
                current_dir()
            } else {
                driver
                    .entry_file()
                    .parent()
                    .expect("entry_file has no parent")
                    .to_owned()
            }
        });
        if is_stdin {
            dir.join("main")
        } else {
            dir.join(
                driver
                    .entry_file()
                    .file_name()
                    .expect("entry_file has no file name"),
            )
        }
    };

    let feature_set =
        FeatureSet::default().configure(&DIAG_FMT_FEATURE, args.diagnostic_format.into());

    // CompileExporter + DynamicLayoutCompiler + WatchDriver
    let driver = CompileExporter::new(driver).with_exporter(exporter);
    let driver = DynamicLayoutCompiler::new(driver, output_dir).with_enable(args.dynamic_layout);
    let actor = CompileActor::new_with_features(driver, feature_set).with_watch(args.watch);

    utils::async_continue(async move {
        utils::logical_exit(actor.run());
    })
}

/// Read from stdin.
fn read_from_stdin() -> FileResult<Vec<u8>> {
    let mut buf = Vec::new();
    let result = io::stdin().read_to_end(&mut buf);
    match result {
        Ok(_) => (),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => (),
        Err(err) => return Err(FileError::from_io(err, Path::new("<stdin>"))),
    }

    Ok(buf)
}
