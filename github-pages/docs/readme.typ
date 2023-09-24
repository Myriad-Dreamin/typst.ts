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
  
  // math setting
  show math.equation: set text(weight: 400)

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
          #place(right, text(luma(110), it.lang))
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

Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

You can:

- locally run the compilation via `typst-ts-cli` to get a precompiled document,
  - or use `typst-ts-compiler` to build your backend programmatically.
- build your frontend using the lightweight TypeScript library `typst.ts`.
- send the precompiled document to your readers' browsers and render it as HTML elements.

The Typst.ts application is designed to be fast due to the following reasons:

- Precompiled documents are much smaller than their PDF equivalents.
  - For example, a compressed precompiled document is only 35KB while its corresponding PDF is 342KB.
- The renderer has a small code size.
- Typst itself has great performance.

== CLI

Run Compiler Example:

```bash
typst-ts-cli compile --workspace "fuzzers/corpora/math" --entry "fuzzers/corpora/math/main.typ"
```

Help:

```bash
$ typst-ts-cli --help
The cli for typst.ts.

Usage: typst-ts-cli [OPTIONS] [COMMAND]

Commands:
  compile  Run compiler. [aliases: c]
  env      Dump Client Environment.
  font     Commands about font for typst.
  help     Print this message or the help of the given subcommand(s)

Options:
  -V, --version  Print Version
      --VV <VV>  Print Version in format [default: none] [possible values: none, short, full, json, json-plain]
  -h, --help     Print help
```

Compile Help:

```bash
$ typst-ts-cli compile --help
Run compiler.

Usage: typst-ts-cli compile [OPTIONS] --entry <ENTRY>

Options:
  -h, --help  Print help

Compile options:
  -w, --workspace <WORKSPACE>    Path to typst workspace [default: .]
      --watch                    watch mode
      --trace <TRACE>            enable tracing. possible usage: --trace=verbosity={0..3} where verbosity: {0..3} -> {warning, info, debug, trace}
  -e, --entry <ENTRY>            Entry file
      --format <FORMAT>          Output formats, possible values: `json`, `pdf`, `ast`, and `rmp`
  -o, --output <OUTPUT>          Output to directory, default in the same directory as the entry file [default: ]
      --font-path <DIR>          Add additional directories to search for fonts
```

== Renderer Example

```shell
# install simple-http-server or other alternative solutions
$ cargo install simple-http-server
$ simple-http-server -p 20810 --cors ./fuzzers/corpora/
$ simple-http-server -p 20811 --cors ./assets/ --compress=ttf,otf
$ cd packages/typst.ts && yarn install && yarn run build && simple-http-server -p 8075 --index --compress=js,json,otf,css,wasm --coep --coop
```

And open your browser to `http://localhost:8075`.

== Precompiler

The compiler is capable of producing artifact outputs from a Typst project. Thet artifact outputs can be easily distributed to remote endpoints.

== Renderer

The renderer accepts an input in artifact format and renders the document as HTML elements.

Import Typst.ts in your project:

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.ts")[\@myriaddreamin/typst.ts]

  ```javascript
  import { createTypstRenderer } from '@myriaddreamin/typst.ts';
  const renderer = createTypstRenderer();
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[\@myriaddreamin/typst.react]

  ```javascript
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

  ```javascript
  /// component.module.ts
  import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
  ```

  Using directive `typst-document` in your template file.

  ```html
  <typst-document fill="#343541" artifact="{{ artifact }}"></typst-document>
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.vue3")[\@myriaddreamin/typst.vue3]

  Coming soon.
