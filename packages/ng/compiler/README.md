# @myriaddreamin/reflexo-typst-compiler

Unified compiler facade for Typst documents.

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

### CLI

The CLI backend is explicit only:

```ts
import { createCliCompiler } from '@myriaddreamin/reflexo-typst-compiler/cli';

const compiler = await createCliCompiler({
  cli: {
    command: 'typst-ts-cli',
  },
});

const artifact = await compiler.vector({
  workspace: process.cwd(),
  mainFilePath: './main.typ',
});
```

Use this backend for build-tool integration or fallback flows where spawning `typst-ts-cli` is acceptable.
