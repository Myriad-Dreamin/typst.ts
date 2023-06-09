# Typst.ts

<p align="center">
  <a href="https://github.com/Myriad-Dreamin/typst.ts/actions/workflows/ci.yaml">
    <img alt="typst_ts::ci" src="https://github.com/Myriad-Dreamin/typst.ts/actions/workflows/ci.yaml/badge.svg"/>
  </a>
  <a href="https://github.com/Myriad-Dreamin/typst.ts/blob/main/LICENSE">
    <img alt="Apache-2 License" src="https://img.shields.io/badge/license-Apache%202-brightgreen"/>
  </a>
</p>

Typst.ts allows you to independently run the Typst compiler and exporter (renderer) in your browser.

![react-demo](https://user-images.githubusercontent.com/35292584/233788011-bd3456e7-6ca2-4567-a5b8-42a65fcb88a5.png)

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

### Prerequisite

- The font assets for Typst.ts are not included in this repository. See [Download Font Assets](./docs/download-font-assets.md) for more information.

### Installation

Download the latest release from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases).

### CLI

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

### Example: Render Typst document in browser (build from source with/without wasm-pack)

Note: see [Troubleshooting WASM Build](docs/troubleshooting-wasm-build.md) for (especially) **Arch Linux** users.

```shell
$ cd packages/typst.ts && yarn install && yarn run build && yarn run link:local; cd ../..
$ cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/
```

And open your browser to `http://localhost:20810/core/`.

You can also run `yarn run build-wrapper` instead of `yarn run build && yarn run link:local` to avoid building the WASM modules from source.

### Example: generate documentation site for packages developers.

- Link [typst-doc](https://github.com/Mc-Zen/typst-doc) by `typst-ts-cli package link --manifest ./typst.toml`.

- Generate documentation by `typst-ts-cli package doc --manifest ./contrib/templates/typst-ts-templates/typst.toml`.

### Concept: Precompiler

The compiler is capable of producing artifact outputs from a Typst project. Thet artifact outputs can be easily distributed to remote endpoints.

### Concept: Renderer

The renderer accepts an input in artifact format and renders the document as HTML elements.

Import Typst.ts in your project:

- Using [@myriaddreamin/typst.ts][npm::typst.ts]

  ```typescript
  import { createTypstRenderer } from '@myriaddreamin/typst.ts';
  const renderer = createTypstRenderer();
  ```

- Using [@myriaddreamin/typst.react][npm::typst.react]

  ```typescript
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

- Using [@myriaddreamin/typst.angular][npm::typst.angular]

  In the module file of your awesome component.

  ```typescript
  /// component.module.ts
  import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
  ```

  Using directive `typst-document` in your template file.

  ```html
  <typst-document fill="#343541" artifact="{{ artifact }}"></typst-document>
  ```

- Using [@myriaddreamin/typst.vue3][npm::typst.vue3]

  Coming soon.

[npm::typst.ts]: https://www.npmjs.com/package/@myriaddreamin/typst.ts
[npm::typst.react]: https://www.npmjs.com/package/@myriaddreamin/typst.react
[npm::typst.angular]: https://www.npmjs.com/package/@myriaddreamin/typst.angular
[npm::typst.vue3]: ./packages/typst.vue3/README.md
