#import "/docs/cookery/book.typ": book-page, cross-link, heading-reference
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Compiler in Rust")

#let compile-middleware = ```rs trait CompileMiddleware```
#let compiler-trait = ```rs trait Compiler```

#include "../claim.typ"

The compiler services help you build a precompiler CLI or an incremental compilation server for #term.vector-format.
- #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa] precompiles (prepares) artifacts for static websites.
- #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/tool/preview.rs")[typst-preview] compiles and streams typst document data to a web browser to provide fast preview of typst documents.

#let sub = heading-reference[== (Archived) The Rust Compiler Library in v0.5.0]
*Note: the following content is for typst.ts >=v0.6.0. To use rust library in \<v0.6.0, check #cross-link("/guide/compiler/service.typ", reference: sub)[the section.]*

The unofficial compiler library provides the world implementation for the official #link("https://github.com/typst/typst")[typst]. To help parallelize the compilation, it provides a ```rs Universe``` that spawns multiple ```rs World``` instances. They are references to the resources to be used by the typst compiler. You can run a compiler task with the `world` reference in another thread:

```rust
std::thread::spawn(move || {
    let doc = typst::compile(&world)?;
});
```

== Importing the Crate

Adding the following to your `Cargo.toml`:

```toml
[dependencies]
reflexo-typst = { version = "0.x.y", features = ["system"] }
```

The example of using the crate natively is the #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/typst.node")[typst-ts-node-compiler].

Usually, we use the `reflexo-typst` crate to build native tools, but it can also compiled to be run in browser, by changing the features to `browser`:

```toml
reflexo-typst = { version = "0.x.y", features = ["browser"] }
```

The example of using the crate in browser is the #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/compiler")[typst-ts-web-compiler].

It can also be compiled as a #link("https://typst.app/docs/reference/foundations/plugin/")[Wasm typst plugin] and be loaded into the typst compiler itself, by changing the features to an empty list:

```toml
reflexo-typst = { version = "0.x.y", features = [] }
```

The example of using the crate as a typst plugin is the _embedded typst_, the #link("https://github.com/typst-doc-cn/tutorial/tree/main/crates/embedded-typst")[Rust part] and the #link("https://github.com/typst-doc-cn/tutorial/blob/main/typ/embedded-typst/example.typ")[typst part].

== Building a Universe

To align the CLI flags with `typst-cli`, we directly provide a convenient struct `CompileOnceArgs` to directly resolve a universe from system arguments.

First, parse the system arguments:

```rust
let args = CompileOnceArgs::parse();
```

Then, simply resolve the universe from the system arguments:

```rust
let verse = args.resolve_system()?;
```

You can also extend the CLI flags with the power of `clap`. For example, #link("https://github.com/Myriad-Dreamin/tinymist/tree/main/crates/crityp")[`crityp`], a benchmark tool for typst scripts, has an additional argument `--bench-output` to specify the output directory of the benchmark results:

```rs
/// Common arguments of crityp benchmark.
#[derive(Debug, Clone, Parser, Default)]
pub struct BenchArgs {
    /// Arguments for compiling the document once, compatible with `typst-cli compile`.
    #[clap(flatten)]
    pub compile: CompileOnceArgs,

    /// Path to output file for benchmarks
    #[clap(long, default_value = "target/crityp")]
    pub bench_output: String,
}
```

More topics about building:
- Configuring fonts: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist-world/src/font/mod.rs")[Font Searchers and Resolvers]
- Configuring packages: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist-package/src/lib.rs")[Package Registries]

== Spawning Worlds for Tasks

The universe provides a synchronous view of compiler resources. It is easy to be modified for incremental compilations, at the cost of not being ensured to be `Sync`. To start (compiler) tasks, you need to spawn a `World` instance from the universe. The `World` instance is `Send` and `Sync`, so you can use it in another thread.

```rust
let world = verse.snapshot();
// in current thread
let doc = typst::compile(&world)?;
// the snapshot is Send + Sync
std::thread::spawn(move || {
    let doc = typst::compile(&world)?;
});
```

Sometimes, you would like to start a task that compiles a different document.

```rust
let world = verse.snapshot_with(Some(reflexo_typst::TaskInputs {
    ..Default::default()
}));
```

You can change either the `entry` (the entry file) or `inputs` (the `sys.inputs` in typst documents).

== Spawning a World with Another Entry

The `entry` field has `EntryState` type, and you can get and mutate the entry in the current world by:

```rust
let entry = verse
  .entry_state()
  .select_in_workspace(Path::new("/main.typ"));
let world = verse.snapshot_with(Some(TaskInputs {
  entry: Some(entry),
  ..Default::default()
}));
```

There is also a fallible version of `select_in_workspace` to process paths from user input:

```rust
let another_entry = current_dir()?.join("main.typ");
let entry = verse
    .entry_state()
    .try_select_path_in_workspace(&another_entry)?;
let world = verse.snapshot_with(Some(TaskInputs {
    entry,
    ..Default::default()
}));
```

Noted that `another_entry` is required to be an absolute path, so that we can check and select the entry file without ambiguity.

There are also constructors for `EntryState`, reflecting the possible state of the entry (a root, and a main file):
- `EntryState::new_rooted_by_parent(entry)`. It accepts an absolute path to the entry file, and sets typst root to the parent directory. The typst document cannot access the files outside of the parent directory (the root).
- `EntryState::new_rooted(root, main)`. It accepts a path to the root directory and a path _relative to_ the root.
- `EntryState::new_rootless(main)`. It accepts an absolute path to the entry file, and the typst document cannot access any other files (other than package files).
- `EntryState::new_workspace(root)`. It accepts root directory, but the entry file is still not determined. You can use `select_in_workspace` and its variants to select the entry file later.
- `EntryState::new_detached()`, where neither the root nor the entry file are determined.

== Spawning a World with Another `sys.inputs`

You can create a new `sys.inputs` either from the `dict!` macro:

```rust
let world = verse.snapshot_with(Some(TaskInputs {
    inputs: Some(Arc::new(LazyHash::new(dict! {
        "my-target" => "markdown"
    }))),
    ..Default::default()
}));
```

or from string pairs:

```rust
let pairs = [("my-target", "markdown")].map(|(k, v)| (k.into(), v.into_value()));
let inputs = Arc::new(LazyHash::new(pairs.into_iter().collect()));
```

== Running the Compilation

You can use the official `typst::compile` to compile a typst document:

```rs
let result = typst::compile::<reflexo_typst::TypstPagedDocument>(&world)?;
```

By default, it is targeting `paged`, i.e., the `sys.target() == "paged"` is true. If you would like to compile the document targeting `html`, you could further modify the `world` for compilation:

```rs
let world = world.html_task();
let result = typst::compile::<reflexo_typst::TypstHtmlDocument>(world.as_ref())?;
```

== Watching (Incremental) Compilation

The `CompileActor` is a wrapper around the universe that provides a convenient way to run the watch compilation loop. The server watches for filesystem changes and compiles them again on demand.

```rs
let (intr_tx, intr_rx) = tokio::sync::mpsc::unbounded_channel();
let actor = reflexo_typst::CompileActor::new(verse, intr_tx, intr_rx).with_watch(args.watch);
tokio::spawn(actor.run());
```

*Note: it is not as stable as the above other APIs and may change in future.*

The `intr_tx` can be used for sending interrupts to the server.

```rs
pub enum Interrupt {
    /// Compile anyway.
    Compile,
    /// Memory file changes.
    Memory(MemoryEvent),
    /// File system event.
    Fs(FilesystemEvent),
    /// Request compiler to stop.
    Settle(oneshot::Sender<()>),
    // ... and others
}
```

== Concurrent Compilation

A concurrent compiler compiles multiple documents at the same time. This is not exposed yet, but you can check the #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/tool/testing.rs")[tinymist test].

```rs
let result = start_project(verse, None, move |c, mut i, next| {
    if let Interrupt::Compiled(artifact) = &mut i {
        let files = artifact.documents();
        let res = test_once(&world, &files);
    }

    // Notifies all the dependences touched by `test_once`.
    next(c, i)
});
```

== Compiling with Memory Shadows

You can shadow paths to avoid filesystem accesses, using the `ShadowApi`. For example, shadowing the main (entry) file using `map_shadow_by_id`:

```rs
let source = Source::new(verse.main_id().unwrap(), "Hello World.".into());
verse
    .map_shadow_by_id(source.id(), Bytes::from_string(source.text().to_owned()))?;
```

The shadow has two layers:
- `{map,unmap}_shadow`: to map/unmap resources on *absolute* paths about filesystem.
- `{map,unmap}_shadow_by_id`: to map/unmap resources on virtual file ids.

== Revising the Universe

You can revise the universe, which generates new revisions about view of resources:

```rs
verse.increment_revision(|verse| {
  verse.vfs().invalidate_path(_);
  verse.vfs().invalidate_file_id(_);

  verse.vfs().reset_shadow();
  verse.vfs().map_shadow(_, _);
  verse.vfs().unmap_shadow(_);
  verse.vfs().map_shadow_by_id(_, _);
  verse.vfs().unmap_shadow_by_id(_);

  verse.vfs().notify_fs_changes(_);

  verse.set_fonts(_);
  verse.set_package(_);
  verse.set_inputs(_);
  verse.set_entry_file(_);
  verse.mutate_entry(_);
});
```

New revisions *may be* created if the resources are changed by the callback function.

== Handling Diagnostics

We provide a `print_diagnostics` function to print the diagnostics:

```rs
let res = typst::compile::<TypstPagedDocument>(&world);
let errors = res.output.err();
let diag = res.warnings.iter().chain(errors.iter().flatten());
let _ = print_diagnostics(&world, diag, reflexo_typst::DiagnosticFormat::Human);
```

It accepts a `reflexo_typst::DiagnosticFormat` to specify the format of the output, either `Human` to pretty print the output or `Short` to print suitable to be reidentified by editors or other tools.

== Cache Eviction

You must carefully evcit cache to avoid memory leak using `CompilerUniverse::evict`, `10` is suggested after each compilation.

```rs
verse.evict(10);
```

== Resource Reloading

Also, you have to reset the universe to react filesystem changes if necessary:

```rs
verse.reset();
```

Noted that `CompilerUniverse::reset` is a heavy operation, and it will reset all the caches and resources. It is not recommended to call it frequently. Insteadly, fined-grained watch compilation like `CompileActor` is suggested.

== Rendering

After compilation, you can do rendering with the artifacts, the `reflexo_typst::TypstPagedDocument` or `reflexo_typst::TypstHtmlDocument`.

It is pretty easy to get such artifacts if you can access the universe directly, using the `typst::compile`. To customized the way of rendering in watch compilation, passing a compilation handler to `reflexo_typst::CompileActor::new_with(opts)`.

== Summary

Gather all the information above, a minimal complete example of using the compiler library is like this:

```rust
let verse = CompileOnceArgs::parse().resolve_system()?;
let doc = typst::compile(&verse.snapshot())?;
verse.evict(10);
```

Or incrementally:

```rs
let (intr_tx, intr_rx) = tokio::sync::mpsc::unbounded_channel();
let actor = reflexo_typst::CompileActor::new(verse, intr_tx, intr_rx).with_watch(true);
tokio::spawn(actor.run());
```

== (Archived) The Rust Compiler Library in v0.5.0

*Note: the following content is for typst.ts \<v0.6.0*

=== Creating and Using a `TypstSystemUniverse` Instance
Note: The ```rs struct TypstSystemUniverse``` can create multiple snapshots at the same time, ```rs struct TypstSystemWorld```, implementing ```rs trait typst::World```.

Example: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/9a4f0537f7d8443b3920a27cabc51bb5ea64ee0a/cli/src/compile.rs#L30")[fn create_driver in compile.rs]

```rs
let verse = TypstSystemUniverse::new(CompileOpts {
  root_dir: workspace_dir.clone(),
  font_paths: args.font.paths.clone(),
  with_embedded_fonts: EMBEDDED_FONT.to_owned(),
  ..CompileOpts::default()
})
.unwrap_or_exit();

// usage
let mut tracer = Tracer::default();
typst::compile(&verse.snapshot(), tracer);
```

=== Creating and Using a `PureCompiler` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct PureCompiler``` implements #compiler-trait. #linebreak()
]

Example:

```rs
std::marker::PhantomData.compile(
  &verse.snapshot(), &mut Default::default());
```

=== Creating and Using a `CompileExporter` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct CompileExporter``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct CompileExporter``` derives #compiler-trait. #linebreak()
]

Retrieve an exporter instance that is executed on each successful compilation (more useful for incremental compilation).

```ts
let driver = CompileExporter::default()
  .with_exporter(exporter)
```

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/export.rs")[exporter.rs] for usage of the exporter feature.

Glance at current available exporters:

```rs
type WithAst = reflexo_typst::AstExporter;
type WithPdf = reflexo_typst::PdfDocExporter;
type WithSvg = reflexo_typst::PureSvgExporter;
type WithSvgHtml = reflexo_typst::SvgHtmlExporter<DefaultExportFeature>;
type WithSIR = reflexo_typst::SvgModuleExporter;
type WithText = reflexo_typst::TextExporter;
```

=== Creating and Using a `DynamicLayoutCompiler` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct DynamicLayoutCompiler``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct DynamicLayoutCompiler``` derives #compiler-trait. #linebreak()
]

Enable dynamic layout based on a #compiler-trait.

```rs
let driver = DynamicLayoutCompiler::new(driver, output_dir);
```

=== Creating and Using a `CompileActor` Instance

Specifical for incremental compilation (Specifically, it watches files and compiles on demand) based on some universe instance.

Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/tool/preview.rs")[use of struct CompileActor in tool/preview.rs in tinymist]

```rs
let (intr_tx, intr_rx) = mpsc::unbounded_channel();
let actor = CompileActor::new(verse,
  intr_tx, intr_rx).with_watch(Some(handle.clone()));
```

Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/actor/typ_client.rs")[use of `intr_tx` in actor/typ_client.rs in tinymist]

Access the service of the `CompileActor` instance.

```rs
/// Updates the overlay layer of VFS (Virtual File System)
let _ = self.intr_tx.send(Interrupt::Memory(event));
/// Reads the snapshot of the current compilation
let (tx, rx) = oneshot::channel();
self.intr_tx.send(Interrupt::SnapshotRead(tx))?;
let snapshot = rx.await
```

// todo: reporter option
// === Example: use a lambda (closure) exporter

// Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/tool/preview.rs")[fn create_driver in compile.rs]

// ```rs
// let driver = CompileExporter::new(compiler_driver).with_exporter(
//   move |_world: &dyn World, doc: Arc<Document>| {
//     let _ = doc_sender.send(Some(doc)); // it is ok to ignore the error here
//     let _ = renderer_sender.send(RenderActorRequest::RenderIncremental);
//     Ok(())
//   },
// );
// ```
