# typst.ts

[Markdown](https://github.com/Myriad-Dreamin/typst.ts/blob/main/README.md) | [typst](./docs/cookery/introduction.typ) |
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

`typst.ts` is a project dedicated to bring the power of
[typst](https://github.com/typst/typst) to the world of JavaScript. In short, it
provides an `typst::World` implementation and several exporters to help compile
and render your Typst document typically inside _Browser Environment_. In the scope of server-side rendering
collaborated by
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

The _Form2: Vector Format_ is developed specially for typst documents, and it has several fancy features:

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/docs/ir-features.dark.artifact.svg">
    <img width="100%" alt="Data Flow" src="https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/docs/ir-features.artifact.svg"/>
  </picture>
</p>

So with _Form2_, you can continue rendering the document in different ways:

##### Static but <ins>responsive</ins> rendering

Example Application: [single-file](https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html), [typst-book](https://github.com/Myriad-Dreamin/typst-book) and [hexo-renderer-typst](https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/hexo-renderer-typst)

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

### Installation (CLI)

Install latest CLI of typst.ts via cargo:

```shell
cargo install --locked --git https://github.com/Myriad-Dreamin/typst.ts typst-ts-cli
```

Or Download the latest release from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases).

### Installation (Packages)

The JavaScript packages are published on [npm](https://www.npmjs.com/).

- Core (Wrapper) Library: [@myriaddreamin/typst.ts][npm::typst.ts]

- React Library: [@myriaddreamin/typst.react][npm::typst.react]

- Angular Library: [@myriaddreamin/typst.angular][npm::typst.angular]

- (Internal) Web compiler WASM module:
  [@myriaddreamin/typst-ts-web-compiler](https://www.npmjs.com/package/@myriaddreamin/typst-ts-web-compiler)

- (Internal) Renderer WASM module:
  [@myriaddreamin/typst-ts-renderer](https://www.npmjs.com/package/@myriaddreamin/typst-ts-renderer)

The rust crates are not published on [crates.io](https://crates.io/) yet, since
it has the git dependency on [typst](https://github.com/typst/typst).

- Core Library: [typst-ts-core](./core/)

- Compiler Library: [typst-ts-compiler](./compiler/)

- CLI as a Library: [typst-ts-cli](./cli/)

### Installation (All-in-one Bundle)

Download the latest bundle file from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases).

### Documentation

See [Documentation](https://myriad-dreamin.github.io/typst.ts/cookery).

### Templates

Please check [Templates](./templates) and usage in [Get Started](https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html).

### Minimal Example

Note: In default, `all-in-one.bundle.js` will download the font assets from
GitHub in browser, so you need to connect to the Internet.

Download `all-in-one.bundle.js` from [GitHub Releases](https://github.com/Myriad-Dreamin/typst.ts/releases) and start a local server with following
content of `index.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Svg Document</title>
    <script type="module" src="http://localhost:12345/all-in-one.bundle.js" id="typst"></script>
  </head>
  <body>
    <textarea id="input" style="width: 100%"></textarea>
    <div id="content"></div>
    <script>
      const input = document.getElementById('input');
      input.value = 'Hello, Typst!';
      document.getElementById('typst').addEventListener('load', function () {
        const compile = function (mainContent) {
          $typst.svg({ mainContent }).then(svg => {
            console.log(`rendered! SvgElement { len: ${svg.length} }`);
            // append svg text
            document.getElementById('content').innerHTML = svg;
          });
        };
        input.oninput = () => compile(input.value);
        compile(input.value);
      });
    </script>
  </body>
</html>
```

And you will see the result.

You can also load the all-in-one bundle file and wasm modules from [jsdelivr](https://www.jsdelivr.com/):

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Svg Document</title>
    <script
      type="module"
      src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one-lite.bundle.js"
      id="typst"
    ></script>
  </head>
  <body>
    <textarea id="input" style="width: 100%"></textarea>
    <div id="content"></div>
    <script>
      const input = document.getElementById('input');
      input.value = 'Hello, Typst!';
      document.getElementById('typst').addEventListener('load', function () {
        $typst.setCompilerInitOptions({
          getModule: () =>
            'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
        });
        $typst.setRendererInitOptions({
          getModule: () =>
            'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
        });

        const compile = function (mainContent) {
          $typst.svg({ mainContent }).then(svg => {
            console.log(`rendered! SvgElement { len: ${svg.length} }`);
            // append svg text
            document.getElementById('content').innerHTML = svg;
          });
        };
        input.oninput = () => compile(input.value);
        compile(input.value);
      });
    </script>
  </body>
</html>
```

### Develop projects along with a local built typst.ts

You can put your owned projects under the `projects` folder, and that yarn workspace will
automatically identify your project. We recommend you to use [git](https://git-scm.com/), [Yarn](https://yarnpkg.com/), and
[turbo](https://turbo.build/) to manage your projects.

##### Example: link a project by git submodule

To develop core external projects, e.g. `typst-preview`, you could initialize them
by command:

```shell
git submodule update --init --checkout projects/typst-preview
```

##### Example: build and run

Ensured that you have [built typst.ts from
source](#build-from-source-and-check), you can build and run the project by
(typst-preview as an example):

```shell
# install dependencies for project
yarn install --pure-lockfile
# build typst-preview and its dependencies
turbo build --filter=typst-preview
@myriaddreamin/typst-ts-renderer:build: cache hit, replaying logs bc0a0b151bd8eb6d
@myriaddreamin/typst.ts:build: cache hit, replaying logs 729cb43a3242b80
typst-preview-frontend:build: cache miss, executing 5ae30471e8957877
typst-preview-frontend:build: ...
typst-preview-frontend:build: ✓ built in 1.25s
typst-preview-frontend:build: Done in 4.57s.
typst-preview:build: cache miss, executing a1bd8ca8233f8a0c
typst-preview:build: ...
typst-preview:build: ✓ built in 1.01s
typst-preview:build: Done in 3.73s.
```

The project (typst-preview as an example) will cache and use the local built packages.

### Build from source and check

Note: you could build from source with/without wasm-pack.

Note: see [Troubleshooting WASM Build](docs/troubleshooting-wasm-build.md) for (especially) **Arch Linux** users.

Note: Since we use turborepo for `>=v0.4.0` development, if you are the earlier developer of `typst.ts`, please clean up all of your node_modules and dist folders before running the commands.

```shell
# build all of typescript packages
$ yarn install && yarn run build:pkg
# compile typst document for demo
$ cargo run --bin typst-ts-dev-server -- compile --compiler debug corpus --cat skyzh-cv
# start a local server
$ cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/
```

And open your browser to `http://localhost:20810/`.

You can also run `yarn run build:core` instead of `yarn run build:pkg` to build
core library (`@myriaddreamin/typst.ts`) and avoid building the WASM modules from source.

<!-- ### Example: generate documentation site for packages developers.

- Link [typst-doc](https://github.com/Mc-Zen/typst-doc) by `typst-ts-cli package link --manifest ./typst.toml`.

- Generate documentation by `typst-ts-cli package doc --manifest ./contrib/templates/typst-ts-templates/typst.toml`. -->

##### Hot Reload

To develop typst.ts with its Wasm renderer, you can run the following command:

```bash
cargo run --bin typst-ts-dev-server -- watch renderer
# or run with yarn script
yarn watch:renderer
```

And open your browser to `http://localhost:20810/`.

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
  import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
  const mainContent = 'Hello, typst!';

  console.log(await $typst.svg({ mainContent }));
  ```

  Specify correct path to the wasm modules if it complains.

  ```typescript
  $typst.setCompilerInitOptions({ getModule: ... });
  $typst.setRendererInitOptions({ getModule: ... });
  ```

  The path option is likely required in browser but not in node.js.

  Further reading: [Get Started with Typst.ts](https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html)

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
