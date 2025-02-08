use std::borrow::Cow;
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

use reflexo_typst::config::entry::{EntryOpts, MEMORY_MAIN_ENTRY};
use reflexo_typst::config::CompileOpts;
use reflexo_typst::path::PathClean;
use reflexo_typst::DynSystemComputation;
use reflexo_typst::{
    CompilationHandle, CompileActor, CompileServerOpts, CompilerFeat, DynComputation, EntryManager,
    EntryReader, ShadowApi, TypstSystemUniverse, WorldComputeGraph,
};
use tokio::sync::mpsc;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Dict, IntoValue};

use crate::font::fonts;
use crate::{
    utils::{self, UnwrapOrExit},
    CompileArgs, CompileOnceArgs,
};

pub fn resolve_universe(args: CompileOnceArgs) -> TypstSystemUniverse {
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

    let verse = TypstSystemUniverse::new(CompileOpts {
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

    let verse = if is_stdin {
        let mut verse = verse;

        let entry = verse
            .entry_state()
            .select_in_workspace(MEMORY_MAIN_ENTRY.vpath().as_rooted_path());
        verse.mutate_entry(entry).unwrap();

        let src = read_from_stdin()
            .map_err(|err| {
                clap::Error::raw(
                    clap::error::ErrorKind::Io,
                    format!("read from stdin failed: {err}\n"),
                )
                .exit()
            })
            .unwrap();

        verse
            .map_shadow_by_id(*MEMORY_MAIN_ENTRY, Bytes::new(src))
            .map_err(|err| {
                clap::Error::raw(
                    clap::error::ErrorKind::Io,
                    format!("map stdin failed: {err}\n"),
                )
                .exit()
            })
            .unwrap();

        verse
    } else {
        verse.with_entry_file(entry_file_path)
    };

    verse
}

pub fn compile_export(args: CompileArgs, exporter: DynSystemComputation) -> ! {
    let (intr_tx, intr_rx) = mpsc::unbounded_channel();

    let verse = resolve_universe(args.compile);

    let handle = Arc::new(CompileHandler { exporter });

    let actor = CompileActor::new_with(
        verse,
        intr_tx,
        intr_rx,
        CompileServerOpts {
            compile_handle: handle,
            ..Default::default()
        },
    )
    .with_watch(args.watch);

    utils::async_continue(async move {
        utils::logical_exit(actor.run().await.unwrap_or_exit());
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

pub struct CompileHandler<F: CompilerFeat> {
    exporter: DynComputation<F>,
}

impl<F: CompilerFeat + 'static> CompilationHandle<F> for CompileHandler<F> {
    fn status(&self, _revision: usize, _rep: reflexo_typst::CompileReport) {}

    fn notify_compile(&self, g: &Arc<WorldComputeGraph<F>>) {
        let res = (self.exporter)(g);
        if let Err(err) = res {
            eprintln!("export failed: {err}");
        }
    }
}
