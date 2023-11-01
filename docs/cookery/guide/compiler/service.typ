#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Compiler Service")

= Compiler Service

#let compile-middleware = ```rs trait CompileMiddleware```
#let compiler-trait = ```rs trait Compiler```
#let world-exporter = ```rs trait WorldExporter```

The compiler services help you build a precompiler CLI or an incremental compilation server Program for #term.vector-format.

== Create and use a `TypstSystemWorld` instance
Note: The ```rs struct TypstSystemWorld``` implements ```rs trait typst::World```.

Example: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/compile.rs#L17")[fn create_driver in compile.rs]

```rs
let world = TypstSystemWorld::new(CompileOpts {
  root_dir: workspace_dir.clone(),
  font_paths: args.font.paths.clone(),
  with_embedded_fonts: EMBEDDED_FONT.to_owned(),
  ..CompileOpts::default()
})
.unwrap_or_exit();

// usage
let mut tracer = Tracer::default();
typst::compile(&world, tracer);
```

== Create and use a `CompileDriver` instance

#[
  #set par(justify: false)
  Note: The ```rs struct CompileDriver``` implements #compiler-trait.
]

The compile driver holds more state for convenient usage.

```rs
let compile_driver = CompileDriver {
  world,
  entry_file: entry_file_path.to_owned(),
}
```

=== Example: get main id of the current entry file

```rs
let main_id = compile_driver.main_id();
```

=== Example: compile document

```rs
let document = compile_driver.compile().unwrap();
```

=== Example: query document

```rs
let selector = "figure".into_owned();
let contents = compile_driver.query(
  selector, document).unwrap();
```

== Create and use a `CompileExporter` instance

#[
  #set par(justify: false)
  Note: The ```rs struct CompileExporter``` implements #compiler-trait. #linebreak()
  Note: The ```rs struct CompileExporter``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct CompileExporter``` implements #world-exporter.
]

Retrieve an exporter instance that is executed on each sucessful compilation (more useful for incremental compilation).

```ts
let driver = CompileExporter::new(compiler_driver)
  .with_exporter(exporter)
```

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/cli/src/export.rs")[exporter.rs] for usage of the exporter feature.

Glance at current available exporters:

```rs
type Ast = typst_ts_ast_exporter::AstExporter;
type Json<T> = typst_ts_serde_exporter::JsonExporter<T>;
type Pdf = typst_ts_pdf_exporter::PdfDocExporter;
type Rmp<T> = typst_ts_serde_exporter::RmpExporter<T>;
type Svg = typst_ts_svg_exporter::PureSvgExporter;
type SvgHtml = typst_ts_svg_exporter::SvgExporter<DefaultExportFeature>;
type SIR = typst_ts_svg_exporter::SvgModuleExporter;
```

=== Example: use a lambda (closure) exporter

Example: #link("https://github.com/Enter-tainer/typst-preview/blob/main/src/actor/typst.rs")[fn create_driver in compile.rs]

```rs
let driver = CompileExporter::new(compiler_driver).with_exporter(
  move |_world: &dyn World, doc: Arc<Document>| {
    let _ = doc_sender.send(Some(doc)); // it is ok to ignore the error here
    let _ = renderer_sender.send(RenderActorRequest::RenderIncremental);
    Ok(())
  },
);
```

== Create and use a `DynamicLayoutCompiler` instance

#[
  #set par(justify: false)
  Note: The ```rs struct CompileExporter``` implements #compiler-trait. #linebreak()
  Note: The ```rs struct DynamicLayoutCompiler``` implements #compile-middleware. #linebreak()
  Note: The ```rs struct DynamicLayoutCompiler``` implements #world-exporter.
]

Enable dynamic layout based on a #compiler-trait.

```rs
let driver = DynamicLayoutCompiler::new(
  driver, output_dir)
  .with_enable(true /* whether enabled dynamic layout */);
```

== Create and use a `CompileActor` instance

Specifical for incremental compilation based on some #world-exporter instance.

```rs
let driver = DynamicLayoutCompiler::new(
  driver, output_dir).with_enable(true /* whether enabled */);
```

Example: #link("https://github.com/Enter-tainer/typst-preview/blob/main/src/actor/typst.rs")[struct TypstActor in typst.rs in typst-preview]

```rs
let actor = CompileActor::new(driver, 
  root.as_ref().to_owned()).with_watch(true);
```

Example #link("https://github.com/Enter-tainer/typst-preview/blob/main/src/actor/typst.rs")[fn TypstActor::run in typst.rs in typst-preview]

Watch input, compile incrementally, and response message:

```rs
pub async fn run(self) {
  let (server, client) = self.inner.split();

  // spawn a watch compile thread
  server.spawn().await;

  debug!("TypstActor: waiting for message");
  let mut client = wrap_client(self.client);
  while let Some(mail) = client.mailbox.recv().await {
    client.process_mail(mail).await;
  }

  info!("TypstActor: exiting");
}
```

== Create your owned compile middleware

Example #link("https://github.com/Enter-tainer/typst-preview/blob/main/src/actor/typst.rs")[struct Reporter in typst.rs in typst-preview]

```rs
impl<C: Compiler> CompileMiddleware for Reporter<C> {
  fn wrap_compile(&mut self) -> SourceResult<Arc<Document>> {
    
    // do someting before each compilation
    // ...

    // trigger real compilation
    let doc = self.inner_mut().compile();
    
    // do someting after each compilation
    // report compilation status
    if let Err(err) = &doc {
      let _ = self.sender.send(EditorActorRequest::CompileStatus(
        CompileStatus::CompileError,
      ));
      log::error!("TypstActor: compile error: {:?}", err);
    } else {
      let _ = self.sender.send(EditorActorRequest::CompileStatus(
        CompileStatus::CompileSuccess,
      ));
    }

    doc
  }
}
```
