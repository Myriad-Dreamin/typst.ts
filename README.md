# typst.ts

[Markdown](https://github.com/Myriad-Dreamin/typst.ts/blob/main/README.md) | [Typst](./docs/cookery/introduction.typ) |
[Online SVG](https://myriad-dreamin.github.io/typst.ts/cookery/) |
[Online Canvas](https://myriad-dreamin.github.io/typst.ts/)

<p align="center">
  <a href="https://github.com/Myriad-Dreamin/typst.ts/actions/workflows/ci.yaml">
    <img alt="typst_ts::ci" src="https://github.com/Myriad-Dreamin/typst.ts/actions/workflows/ci.yaml/badge.svg"/>
  </a>
  <a href="https://github.com/Myriad-Dreamin/typst.ts/blob/main/LICENSE">
    <img alt="Apache-2 License" src="https://img.shields.io/badge/license-Apache%202-brightgreen"/>
  </a>
</p>

`typst.ts` is a project dedicated to bring the power of [Typst](https://github.com/typst/typst) to the world of JavaScript. In short, it provides an `typst::World` implementation and several exporters to help compile and render your Typst document. In the scope of server-side rendering collaborated by
$\textcolor{#3c9123}{\textsf{server}}$ and $\textcolor{#0074d9}{\textsf{browser}}$, there would be a data flow like this:

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/docs/data-flow-standalone.dark.artifact.svg">
    <img width="100%" alt="Data Flow" src="https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/docs/data-flow-standalone.artifact.svg"/>
  </picture>
</p>

Specifically, it first typically presents a typst document in three forms:

- [Form1](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/ts-cli.html): Render to SVG and then embed it as a high-quality vectored image directly.

- [Form2](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/ts-cli.html): Preprocessed to a Vector Format artifact.

- [Form3](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/serverless.html): Manipulate a canvas element directly.

The _Form2: Vector Format_ is developed specially for typst documents, and it has several fancy features: Todo: Visualized Feature:

- Data Sharing across multiple layouts.

- Artifact Streaming.

- Incremental Rendering.

So with _Form2_, you can continue rendeing the document in different ways:

##### Static but <ins>responsive</ins> rendering

Example Application: [single-file](https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html), [typst-book](https://github.com/Myriad-Dreamin/typst-book) and [hexo-renderer-typst](https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/hexo-renderer-typst)

A compressed artifact containing data for different theme and screen settings. The bundle size of artifacts is optimized for typst documents.

##### <ins>Incremental</ins> server-side rendering

Example Application: [typst-preview](https://github.com/Enter-tainer/typst-preview-vscode)

Build a server for compilation with [Compiler Service](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/service.html), streaming the artifact, and render it incrementally.

##### <ins>Serverless</ins> client-side rendering

Example Application: [single-file](https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/compiler.html)

Run the entire typst directly in browser, like [typst.app](https://typst.app).

### Application

- [A Website built with Typst.ts](https://myriad-dreamin.github.io/typst.ts/)

- [Instant VSCode Preview Plugin](https://github.com/Enter-tainer/typst-preview-vscode)

- [typst-book - A simple tool for creating modern online books in pure typst.](https://github.com/Myriad-Dreamin/typst-book)

- [Renderer Plugin for Hexo, a Blog-aware Static Site Generator](https://www.npmjs.com/package/hexo-renderer-typst)

- Renderer/Component Library for [JavaScript](https://www.npmjs.com/package/@myriaddreamin/typst.ts), [React](https://www.npmjs.com/package/@myriaddreamin/typst.react), and [Angular](https://www.npmjs.com/package/@myriaddreamin/typst.angular)

### Installation

Install latest CLI of typst.ts via cargo:

```shell
cargo install --locked --git https://github.com/Myriad-Dreamin/typst.ts typst-ts-cli
```

Or Download the latest release from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases).

### Example: Render Typst document in browser (build from source with/without wasm-pack)

Note: see [Troubleshooting WASM Build](docs/troubleshooting-wasm-build.md) for (especially) **Arch Linux** users.

Note: Since we use turborepo for `>=v0.4.0` development, if you are the earlier developer of `typst.ts`, please clean up all of your node_modules and dist folders before running the commands.

```shell
# Optional: download the font assets if you haven't done so.
$ git submodule update --init --recursive .
# build all of typescript packages
$ yarn install && npx turbo run build
# compile typst document for demo
$ cargo run --bin typst-ts-dev-server -- compile --compiler debug corpus --cat skyzh-cv
# start a local server
$ cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/
```

And open your browser to `http://localhost:20810/`.

You can also run `yarn run build:core` instead of `npx turbo run build` to build
core library (`@myriaddreamin/typst.ts`) and avoid building the WASM modules from source.

### Example: generate documentation site for packages developers.

- Link [typst-doc](https://github.com/Mc-Zen/typst-doc) by `typst-ts-cli package link --manifest ./typst.toml`.

- Generate documentation by `typst-ts-cli package doc --manifest ./contrib/templates/typst-ts-templates/typst.toml`.

### Concept: Precompiler

The precompiler is capable of producing artifact outputs from a Typst project. Thet artifact outputs can be easily distributed to remote endpoints.

Install latest precompiler via cargo:

```shell
cargo install --locked --git https://github.com/Myriad-Dreamin/typst.ts typst-ts-cli
```

Or Download the latest release from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases).

### Concept: Renderer

The renderer accepts an input in artifact format and renders the document as HTML elements.

Import `typst.ts` in your project:

- Using [@myriaddreamin/typst.ts][npm::typst.ts]

  ```typescript
  import { createTypstRenderer } from '@myriaddreamin/typst.ts';
  const renderer = createTypstRenderer();
  await renderer.init();
  const svg = await renderer.runWithSession(async session => {
    // do something with session
  });
  ```

- Using [@myriaddreamin/typst.react][npm::typst.react]

  ```typescript
  import { TypstDocument } from '@myriaddreamin/typst.react';

  export const App = (artifact: Uint8Array) => {
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
