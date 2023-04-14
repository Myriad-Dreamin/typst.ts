# Typst.ts

Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

You can:

+ locally run the compilation via `typst-ts-cli` to get a precompiled document,
  + or use `typst-ts-compiler` to build your backend programmatically.
+ build your frontend using the lightweight TypeScript library `typst.ts`.
+ send the precompiled document to your readers' browsers and render it as HTML elements.

The Typst.ts application is designed to be fast due to the following reasons:
+ Precompiled documents are much smaller than their PDF equivalents.
  + For example, a compressed precompiled document is only 35KB while its corresponding PDF is 342KB.
+ The renderer has a small code size.
+ Typst itself has great performance.

### Prerequisite

+ The font assets for Typst.ts are not included in this repository. See [Download Font Assets](./docs/download-font-assets.md) for more information.

### CLI

Run Compiler Example:

```shell
typst-ts-cli compile --workspace "fuzzers/corpora/math" --entry "fuzzers/corpora/math/math.typ"
```

Help:

```shell
$ typst-ts-cli --help
The cli for typst.ts.

Usage: typst-ts-cli <COMMAND>

Commands:
  compile  Run precompiler. [aliases: c]
  help     Print this message or the help of the given subcommand(s)
```

Compile Help:

```shell
$ typst-ts-cli compile --help
Run precompiler.

Usage: typst-ts-cli compile [OPTIONS] --entry <ENTRY>

Compile options:
  -w, --workspace <WORKSPACE>  Path to typst workspace [default: .]
  -e, --entry <ENTRY>          Entry file
      --format <FORMAT>        Output formats
  -o, --output <OUTPUT>        Output to directory, default in the same directory as the entry file [default: ]
```

### Renderer Example

```shell
$ cd renderer && npm install && npm run build && python -m http.server 8075
```

And open your browser to `http://localhost:8075`.

### Precompiler

The compiler is capable of producing artifact outputs from a Typst project. Thet artifact outputs can be easily distributed to remote endpoints.

### Renderer

The renderer accepts an input in artifact format and renders the document as HTML elements.
