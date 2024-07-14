#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Compiler in Rust")

#let compile-middleware = ```rs trait CompileMiddleware```
#let compiler-trait = ```rs trait Compiler```

The compiler services help you build a precompiler CLI or an incremental compilation server Program for #term.vector-format.

== Create and use a `TypstSystemUniverse` instance
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

== Create and use a `PureCompiler` instance

#[
  #set par(justify: false)
  Note: The ```rs struct PureCompiler``` implements #compiler-trait. #linebreak()
]

Example:

```rs
std::marker::PhantomData.compile(
  &verse.snapshot(), &mut Default::default());
```

== Create and use a `CompileExporter` instance

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
type WithAst = typst_ts_ast_exporter::AstExporter;
type WithPdf = typst_ts_pdf_exporter::PdfDocExporter;
type WithSvg = typst_ts_svg_exporter::PureSvgExporter;
type WithSvgHtml = typst_ts_svg_exporter::SvgExporter<DefaultExportFeature>;
type WithSIR = typst_ts_svg_exporter::SvgModuleExporter;
type WithText = typst_ts_text_exporter::TextExporter;
```

== Create and use a `DynamicLayoutCompiler` instance

#[
  #set par(justify: false)
  Note: The ```rs struct DynamicLayoutCompiler``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct DynamicLayoutCompiler``` derives #compiler-trait. #linebreak()
]

Enable dynamic layout based on a #compiler-trait.

```rs
let driver = DynamicLayoutCompiler::new(driver, output_dir);
```

== Create and use a `CompileActor` instance

Specifical for incremental compilation based on some universe instance.

Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/server/preview_compiler.rs")[struct CompileServer in preview_compiler.rs in typst-preview]

```rs
let (intr_tx, intr_rx) = mpsc::unbounded_channel();
let actor = CompileServerActor::new(verse,
  intr_tx, intr_rx).with_watch(Some(handle.clone()));
```

Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/server/preview_compiler.rs")[fn CompileServer::spawn in preview_compiler.rs in typst-preview]

Watch input, compile incrementally, and response message:

```rs
pub async fn run(self) {
  let intr_tx = self.inner.intr_tx.clone();
  // spawn a watch compile thread
  tokio::spawn(self.inner.spawn());

  debug!("TypstActor: waiting for message");
  let mut client = wrap_client(intr_tx);
  while let Some(mail) = client.mailbox.recv().await {
    client.process_mail(mail).await;
  }

  info!("TypstActor: exiting");
}
```

// todo: reporter option
// === Example: use a lambda (closure) exporter

// Example: #link("https://github.com/Myriad-Dreamin/tinymist/blob/main/crates/tinymist/src/server/preview_compiler.rs")[fn create_driver in compile.rs]

// ```rs
// let driver = CompileExporter::new(compiler_driver).with_exporter(
//   move |_world: &dyn World, doc: Arc<Document>| {
//     let _ = doc_sender.send(Some(doc)); // it is ok to ignore the error here
//     let _ = renderer_sender.send(RenderActorRequest::RenderIncremental);
//     Ok(())
//   },
// );
// ```

=== Adds exporters to a `CompileActor` instance

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
