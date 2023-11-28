## Cetz Editor

This is a sample project to exhibit how to develop along with/without a local built typst.ts.

> You can put your owned projects under the `projects` folder, and that yarn workspace will
> automatically identify your project. We recommend you to use [git](https://git-scm.com/), [Yarn](https://yarnpkg.com/), and
> [turbo](https://turbo.build/) to manage your projects.

There are three ways to import the wasm modules in the sample project (See the [`src/index.mts`](./src/index.mts) file):

1. Use the wasm modules from CDN (default):

```ts
let moduleSource: ModuleSource = 'jsdelivr';
```

2. Bundle the wasm modules via vite:

```ts
// @ts-ignore
import compiler from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
// @ts-ignore
import renderer from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';
```

3. Serve the wasm modules locally via `typst-ts-dev-server` (See [Setup the typst.ts in local](#setup-the-typstts-in-local)):

Please

- ensure that `cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/`
- change `window.$typst$moduleSource` manually in the [`src/index.mts`](./src/index.mts) file:

```ts
window.$typst$moduleSource = 'local';
```

Note that you should align the version of the wasm modules with the version of the typst.ts package:

```json
{
  "peerDependencies": {
    "@myriaddreamin/typst.ts": "0.4.1-rc2",
    "@myriaddreamin/typst-ts-renderer": "0.4.1-rc2",
    "@myriaddreamin/typst-ts-web-compiler": "0.4.1-rc2"
  }
}
```

Otherwise you will get an error like this:

```log
Uncaught (in promise) LinkError: WebAssembly.instantiate(): Import #73 module="wbg" function="__wbindgen_closure_wrapper16065": function import requires a callabl
```

Or this:

```log
panicked at `called Result::unwrap()` on an `Err` value: CheckBytesError(...)
```

### Setup the wasm modules in local

Build the typst.ts project and start a local server:

```bash
# Optional: download the font assets if you haven't done so.
$ git submodule update --init --recursive .
# build all of typescript packages
$ yarn install && yarn run build:pkg
# compile typst document for demo
$ cargo run --bin typst-ts-dev-server -- compile --compiler debug corpus --cat skyzh-cv
# start a local server
$ cargo run --bin typst-ts-dev-server -- run http --corpus ./fuzzers/corpora/
```

### Debug the cetz editor

```bash
# watch and compile typescript
$ npx tsc -w
# start a vite dev server
$ yarn run dev
# open the browser
$ open http://localhost:5173
```

### Caution

This is a quick and dirty project, so you may face some problems when you try to run it. Please feel free to open an issue if you have any questions.

Also feel free to open a PR to help us improve this sample project.
