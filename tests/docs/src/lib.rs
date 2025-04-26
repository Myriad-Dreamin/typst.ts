#![cfg(test)]

use std::{env::current_dir, path::Path, sync::Arc};

use reflexo_typst::{
    print_diagnostics, syntax::Source, Bytes, CompileActor, EntryReader, LazyHash, ShadowApi,
    TaskInputs, TypstHtmlDocument, TypstPagedDocument,
};
use typst::{foundations::dict, foundations::IntoValue};

#[test]
fn test() {
    // todo: prelude it?
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let _universe = args
        .resolve_system()
        .expect("failed to resolve system universe");
}

#[test]
fn test_snapshot() {
    // todo: prelude it?
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let world = verse.snapshot();
    typst::compile::<TypstPagedDocument>(&world)
        .output
        .expect("no errors");
}

#[test]
fn test_snapshot_html() {
    // todo: prelude it?
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let world = verse.snapshot();
    typst::compile::<TypstHtmlDocument>(world.html_task().as_ref())
        .output
        .expect("no errors");
}

#[test]
fn test_snapshot_diag() {
    // todo: prelude it?
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let world = verse.snapshot();
    let res = typst::compile::<TypstPagedDocument>(&world);
    let errors = res.output.err();
    let diag = res.warnings.iter().chain(errors.iter().flatten());
    let _ = print_diagnostics(&world, diag, reflexo_typst::DiagnosticFormat::Human);
}

#[test]
fn test_snapshot_watch() {
    // todo: prelude it?
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let (intr_tx, intr_rx) = tokio::sync::mpsc::unbounded_channel();
    let actor = CompileActor::new(verse, intr_tx, intr_rx).with_watch(true);
    let _spawn_it = || tokio::spawn(actor.run());
}

#[test]
fn test_snapshot_with() {
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let entry = verse
        .entry_state()
        .select_in_workspace(Path::new("/main.typ"));
    let world = verse.snapshot_with(Some(TaskInputs {
        entry: Some(entry),
        ..Default::default()
    }));
    typst::compile::<TypstPagedDocument>(&world)
        .output
        .expect("no errors");
}

#[test]
fn test_snapshot_with_try() {
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let another_entry = current_dir().expect("cwd").join("main.typ");
    let entry = verse
        .entry_state()
        .try_select_path_in_workspace(&another_entry)
        .expect("failed to select path");
    let world = verse.snapshot_with(Some(TaskInputs {
        entry,
        ..Default::default()
    }));
    typst::compile::<TypstPagedDocument>(&world)
        .output
        .expect("no errors");
}

#[test]
fn test_snapshot_with_inputs() {
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let pairs = [("my-target", "markdown")].map(|(k, v)| (k.into(), v.into_value()));
    let world = verse.snapshot_with(Some(TaskInputs {
        inputs: Some(Arc::new(LazyHash::new(pairs.into_iter().collect()))),
        ..Default::default()
    }));
    typst::compile::<TypstPagedDocument>(&world)
        .output
        .expect("no errors");
}

#[test]
fn test_snapshot_with_inputs_macro() {
    use clap::Parser;
    use reflexo_typst::args::CompileOnceArgs;
    let args = CompileOnceArgs::parse_from(["tinymist", "main.typ"]);
    let mut verse = args
        .resolve_system()
        .expect("failed to resolve system universe");
    let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
    verse
        .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))
        .expect("failed to map shadow");
    let world = verse.snapshot_with(Some(TaskInputs {
        inputs: Some(Arc::new(LazyHash::new(dict! {
            "my-target" => "markdown"
        }))),
        ..Default::default()
    }));
    typst::compile::<TypstPagedDocument>(&world)
        .output
        .expect("no errors");
}
