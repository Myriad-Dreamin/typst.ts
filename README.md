# Typst.ts

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
  compile  Run compiler. [aliases: c]
  font     Commands about font for typst.
  help     Print this message or the help of the given subcommand(s)
```

Compile Help:

```shell
$ typst-ts-cli compile --help
Run compiler.

Usage: typst-ts-cli compile [OPTIONS] --entry <ENTRY>

Compile options:
  -w, --workspace <WORKSPACE>  Path to typst workspace [default: .]
  -e, --entry <ENTRY>          Entry file
      --format <FORMAT>        Output formats
  -o, --output <OUTPUT>        Output to directory, default in the same directory as the entry file [default: ]
      --watch                  watch mode
```

### Renderer Example

```shell
$ cd packages/typst.ts && yarn install && yarn run build && python -m http.server 8075
```

And open your browser to `http://localhost:8075`.

### Precompiler

The compiler is capable of producing artifact outputs from a Typst project. Thet artifact outputs can be easily distributed to remote endpoints.

### Renderer

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

[npm::typst.ts]: https://www.npmjs.com/package/@myriaddreamin/typst.ts

[npm::typst.react]: https://www.npmjs.com/package/@myriaddreamin/typst.react
