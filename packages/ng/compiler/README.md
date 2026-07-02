# @myriaddreamin/reflexo-typst-compiler

Unified compiler package for Typst documents.

The package does not bundle every backend into the entry point. Pick a backend explicitly in production, or use `auto` while prototyping:

```ts
import { createCompiler } from '@myriaddreamin/reflexo-typst-compiler';

const compiler = await createCompiler({ backend: 'auto' });
const artifact = await compiler.vector({
  mainFileContent: 'Hello, typst!',
});
```

`auto` selects:

- `node` in Node.js
- `wasm` outside Node.js

It never selects `cli` automatically.
It also never selects `wasm-worker`; worker isolation is an explicit opt-in.

## Backends

### Node.js

Install the peer package:

```bash
npm install @myriaddreamin/reflexo-typst-compiler @myriaddreamin/typst-ts-node-compiler
```

```ts
import { createNodeCompiler } from '@myriaddreamin/reflexo-typst-compiler/node';

const compiler = await createNodeCompiler();
const pdf = await compiler.pdf({
  mainFileContent: 'Hello, typst!',
});
```

### Wasm

Install the peer package:

```bash
npm install @myriaddreamin/reflexo-typst-compiler @myriaddreamin/typst.ts
```

```ts
import { createWasmCompiler } from '@myriaddreamin/reflexo-typst-compiler/wasm';

const compiler = await createWasmCompiler({
  wasm: {
    initOptions: {
      getModule: () => '/typst_ts_web_compiler_bg.wasm',
    },
  },
});

const artifact = await compiler.vector({
  mainFileContent: 'Hello, typst!',
});
```

The Wasm backend currently exposes compiler artifacts and PDF export through `@myriaddreamin/typst.ts/compiler`. SVG/HTML rendering should be layered through a renderer package.

The Wasm compiler can rebuild the underlying compiler when providers change:

```ts
await compiler.setFontProvider({
  fonts: [fontBytes],
  loadOptions: { assets: false },
});
await compiler.setAccessModel(accessModel);
await compiler.setPackageProvider(packageRegistry);
```

### Wasm Worker

Use the worker backend when browser compilation should run off the main thread:

```ts
import { createWasmWorkerCompiler } from '@myriaddreamin/reflexo-typst-compiler/wasm-worker';

const compiler = await createWasmWorkerCompiler({
  wasm: {
    initOptions: {
      getModule: () => wasmBytes,
    },
  },
});

const artifact = await compiler.vector({
  mainFileContent: 'Hello, typst!',
});
await compiler.terminate();
```

`wasm-worker` accepts structured-cloneable font providers. Access models and package registries must be created inside a custom worker for now, because their methods cannot cross `postMessage`.

### CLI

The CLI backend is explicit only:

```ts
import { createCliCompiler } from '@myriaddreamin/reflexo-typst-compiler/cli';

const compiler = await createCliCompiler({
  cli: {
    command: 'typst',
    vectorCommand: 'typst-ts-cli',
  },
});

const artifact = await compiler.vector({
  workspace: process.cwd(),
  mainFilePath: './main.typ',
});
```

Use this backend for build-tool integration or fallback flows where spawning CLI processes is acceptable.
The CLI backend uses the official `typst` command for `pdf`, `svg`, and `html` exports. It uses `typst-ts-cli` only for `vector`, because vector artifacts are specific to typst.ts.
