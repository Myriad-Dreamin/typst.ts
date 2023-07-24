
// #import "@preview/canvas:0.1.0": canvas
// #import "/contrib/typst/typst-canvas/lib.typ": canvas
#import "/contrib/typst/diagram.typ": node, arr, commutative_diagram

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

#let data-flow-graph = commutative_diagram(
  node_padding: (70pt, 50pt),
  node((0, 0), [
    Typst Documents
  ]),
  node((0, 2), [
    Preprocessed Artifact
  ]),
  node((1, 1), [
    #link("https://developer.mozilla.org/en-US/docs/Web/SVG")[Svg Document] ( `<svg/>` )
  ]),
  node((2, 1), [
    #link("https://developer.mozilla.org/en-US/docs/Web/HTML/Element/canvas")[Canvas] ( `<canvas/>` )
  ]),
  arr((0, 0), (0, 2), [
    #set text(fill: green)
    `precompile with theme and screen settings`
  ]),
  arr((0, 0), (1, 1), label_pos: 0.8em, {
    set text(fill: green)
    rotate(17deg)[
      `compile to svg`
      #set text(fill: blue)
      #h(-0.5em) $space^dagger$
    ]
  }),
  arr((0, 0), (2, 1), label_pos: -0.6em, curve: -25deg, {
    set text(fill: blue)
    rotate(35deg)[`directly render` #h(-0.5em) $ space^(dagger dot.c dagger.double)$]
  }),
  arr((0, 2), (1, 1), label_pos: -0.8em, {
    set text(fill: blue)
    rotate(-17deg)[`render to svg` #h(-0.5em) $ space^dagger.double$]
  }),
  arr((1, 1), (2, 1), []),
  arr((0, 2), (2, 1), label_pos: 0.6em, curve: 25deg, {
    set text(fill: blue)
    rotate(-35deg)[`render to canvas` #h(-0.5em) $ space^(dagger.double)$]
  }),
)

#figure(
  data-flow-graph,
  caption: [Browser-side module needed: $dagger$: renderer; $dagger.double$: compiler. ],
  numbering: none,
)

Specifically, it supports several typical approaches:

- Statically compile your Typst document to browser-friendly SVG documents, which can be easily embedded in your HTML page.

- Precompile your Typst document to a compressed artifact, allowing to render with speicific theme and screen settings at the browser side.

- Run the typst compiler directly in browser, like #link("https://typst.app")[typst.app].

// Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

// You can:

// - locally run the compilation via `typst-ts-cli` to get a precompiled document,
//   - or use `typst-ts-compiler` to build your backend programmatically.
// - build your frontend using the lightweight TypeScript library `typst.ts`.
// - send the precompiled document to your readers' browsers and render it as HTML elements.

The Typst.ts application is designed to be fast due to the following reasons:

- Precompiled documents are much smaller than their PDF equivalents.
  - For example, a compressed precompiled document is only 35KB while its corresponding PDF is 342KB.
- The renderer module has a small code size.
- Typst itself has great performance.

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
