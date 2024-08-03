#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Compiler in Rust")

#let compile-middleware = ```rs trait CompileMiddleware```
#let compiler-trait = ```rs trait Compiler```

#include "../claim.typ"

The compiler services help you build a precompiler CLI or an incremental compilation server Program for #term.vector-format.

== Creating and Using a `TypstSystemUniverse` Instance
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

== Creating and Using a `PureCompiler` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct PureCompiler``` implements #compiler-trait. #linebreak()
]

Example:

```rs
std::marker::PhantomData.compile(
  &verse.snapshot(), &mut Default::default());
```

== Creating and Using a `CompileExporter` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct CompileExporter``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct CompileExporter``` derives #compiler-trait. #linebreak()
]

Retrieve an exporter instance that is executed on each sucessful compilation (more useful for incremental compilation).

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

== Creating and Using a `DynamicLayoutCompiler` Instance

#[
  #set par(justify: false)
  Note: The ```rs struct DynamicLayoutCompiler``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct DynamicLayoutCompiler``` derives #compiler-trait. #linebreak()
]

Enable dynamic layout based on a #compiler-trait.

```rs
let driver = DynamicLayoutCompiler::new(driver, output_dir);
```

== Creating and Using a `CompileActor` Instance

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

=== Adding Exporters to a `CompileActor` Instance

Example #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/compile.rs")[fn compile_export in compile.rs in typst-ts-cli]

```rs
let mut exporters: Vec<DynExporter<CompileSnapshot<_>>> = vec![];

if args.dynamic_layout {
    let driver = DynamicLayoutCompiler::new(
      std::marker::PhantomData, output_dir);
    exporters.push(Box::new(CompileStarter::new(driver)));
}

let actor = CompileActor::new_with(
    verse, intr_tx, intr_rx,
    CompileServerOpts {
        exporter: GroupExporter::new(exporters),
        ..Default::default()
    },
)
.with_enable_watch(args.watch);
```
