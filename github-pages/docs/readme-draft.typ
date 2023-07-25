
// #import "@preview/canvas:0.1.0": canvas
// #import "/contrib/typst/typst-canvas/lib.typ": canvas
#import "data-flow.typ": data-flow-graph

// The project function defines how your document looks.
// It takes your content and some metadata and formats it.
// Go ahead and customize it to your liking!
#let project(title: "", authors: (), body) = {
  // Set the document's basic properties.

  let style_color = rgb("#ffffff")
  set document(author: authors, title: title)
  set page(
    numbering: none, 
    number-align: center,
    height: auto,
    background: rect(fill: rgb("#343541"), height: 100%, width: 100%)
  )
  set text(font: "Linux Libertine", lang: "en", fill: style_color)

  show link: underline
  
  // math setting
  // show math.equation: set text(weight: 400)

  // code block setting
  show raw: it => {
    if it.block {
      rect(
        width: 100%,
        inset: (x: 4pt, y: 5pt),
        radius: 4pt,
        fill: rgb(239, 241, 243),
        [
          #set text(fill: rgb("#000000"))
          #it
        ],
      )
    } else {
      it
    }
  }

  // Main body.
  set par(justify: true)

  body
}

#show: project

Typst.ts is a project dedicated to bring the power of #link("https://github.com/typst/typst")[Typst] to the world of JavaScript. In short, it composes ways to compile and render your Typst document. In the scope of server-side rendering collaborated by #text(fill: green, "server") and #text(fill: blue, "browser"), there would be a data flow like this:

#figure(
  data-flow-graph,
  caption: [Browser-side module needed: $dagger$: compiler; $dagger.double$: renderer. ],
  numbering: none,
)

Specifically, it provides several typical approaches:

- Compile Typst documents to browser-friendly SVG documents.

- Precompile Typst documents to a compressed artifact.

- Run the typst compiler directly in browser, like #link("https://typst.app")[typst.app].

// Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

// You can:

// - locally run the compilation via `typst-ts-cli` to get a precompiled document,
//   - or use `typst-ts-compiler` to build your backend programmatically.
// - build your frontend using the lightweight TypeScript library `typst.ts`.
// - send the precompiled document to your readers' browsers and render it as HTML elements.

Visualized Feature:

- Artifact Streaming

- Incremental Rendering

- Incremental Font Transfer

== Application

- #link("https://myriad-dreamin.github.io/typst.ts/")[A Website built with Typst.ts]

- #link("https://github.com/Enter-tainer/typst-preview-vscode")[Instant VSCode Preview Plugin]

- #link("https://www.npmjs.com/package/hexo-renderer-typst")[Renderer Plugin for Hexo, a Blog-aware Static Site Generator]

- Renderer/Component Library for #link("https://www.npmjs.com/package/@myriaddreamin/typst.ts")[JavaScript], #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[React], and #link("https://www.npmjs.com/package/@myriaddreamin/typst.angular")[Angular]

== Installation

See https://github.com/Myriad-Dreamin/typst.ts/releases for precompiler and npm packages in Usage: Renderer section for renderer.

== Usage

== CLI (Precompiler)


Run Compiler Example:

```shell
typst-ts-cli compile --workspace "fuzzers/corpora/math" --entry "fuzzers/corpora/math/main.typ"
```

Help:

```shell
$ typst-ts-cli --help
The cli for typst.ts.

Usage: typst-ts-cli [OPTIONS] [COMMAND]

Commands:
  compile  Run compiler. [aliases: c]
  completion  Generate shell completion script
  env      Dump Client Environment.
  font     Commands about font for typst.
  help     Print this message or the help of the given subcommand(s)
  package     Commands about package for typst.

Options:
  -V, --version  Print Version
      --VV <VV>  Print Version in format [default: none] [possible values: none, short, full, json, json-plain]
  -h, --help     Print help
```

Compile Help:

```shell
$ typst-ts-cli compile --help
Run compiler.

Usage: typst-ts-cli compile [OPTIONS] --entry <ENTRY>

Compile options:
  -w, --workspace <WORKSPACE>  Path to typst workspace [default: .]
      --watch                  Watch mode
      --dynamic-layout         Generate dynamic layout representation. Note: this is an experimental feature and will be merged as format `dyn-svg` in the future
      --trace <TRACE>          Enable tracing. Possible usage: --trace=verbosity={0..3} where verbosity: {0..3} -> {warning, info, debug, trace}
  -e, --entry <ENTRY>          Entry file
      --format <FORMAT>        Output formats, possible values: `json`, `pdf`, `svg`, `json_glyphs`, `ast`, `ir`, and `rmp`
  -o, --output <OUTPUT>        Output to directory, default in the same directory as the entry file [default: ]
      --font-path <DIR>        Add additional directories to search for fonts
```

Package Help:

```shell
$ typst-ts-cli package --help
Commands about package for typst.

Usage: typst-ts-cli package <COMMAND>

Commands:
  doc     Generate documentation for a package
  help    Print this message or the help of the given subcommand(s)
  link    Link a package to local data path
  list    List all discovered packages in data and cache paths
  unlink  Unlink a package from local data path

Options:
  -h, --help  Print help
```

==== Renderer <renderer-example>

The renderer accepts an input in artifact format and renders the document as HTML elements.

Import Typst.ts in your project:

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.ts")[\@myriaddreamin/typst.ts]

  ```ts
  import { createTypstRenderer } from '@myriaddreamin/typst.ts';
  const renderer = createTypstRenderer();
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[\@myriaddreamin/typst.react]

  ```tsx
  import { TypstDocument } from '@myriaddreamin/typst.react';

  export const App = (artifact: string) => {
    return (
      <div>
        <h1>Demo: Embed Your Typst Document in React</h1>
        <TypstDocument fill="#343541" artifact={artifact} />
      </div>
    );
  };
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.angular")[\@myriaddreamin/typst.angular]

  In the module file of your awesome component.

  ```ts
  /// component.module.ts
  import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
  ```

  Using directive `typst-document` in your template file.

  ```html
  <typst-document fill="#343541" artifact="{{ artifact }}"></typst-document>
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.vue3")[\@myriaddreamin/typst.vue3]

  Coming soon.


== Development (Build from source)

==== Prerequisite

- The font assets for Typst.ts are not included in this repository. See [Download Font Assets](./docs/download-font-assets.md) for more information.

=== Renderer Example

```shell
$ cd packages/typst.ts && yarn install && yarn run build && yarn run link:local; cd ../..
$ cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/
```

And open `http://localhost:8075` in your browser.

You can also run `yarn run build-wrapper` instead of `yarn run build && yarn run link:local` to avoid building the WASM modules from source..
