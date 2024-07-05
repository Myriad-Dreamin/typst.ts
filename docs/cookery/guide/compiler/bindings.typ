#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "Compiler in Wasm (Web)")

#let snippet-source = "https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/contrib/snippet.mts"
#let snippet-lib = link(snippet-source)[`snippet`]

The most simple examples always work with #snippet-lib utility library, an all-in-one library with simplified API interfaces:

```ts
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

Please check #cross-link("/guide/all-in-one.typ")[All-in-one (Simplified) Library for Browsers] for more details.

Quick example for the harder way to serverless compiler:

```ts
import { createTypstCompiler } from '@myriaddreamin/typst.ts';

const mainFilePath = '/main.typ';
const cc /* compiler */ = createTypstCompiler();
await cc.init();
cc.addSource(mainFilePath, 'Hello, typst!');
await cc.compile({ mainFilePath });
```

Note: For #link("https://developer.mozilla.org/en-US/docs/Glossary/Tree_shaking")[_tree-shaking_], you should import it with longer path:

In *ES Module* path:

```ts
import { createTypstCompiler } from '@myriaddreamin/typst.ts/dist/esm/compiler.mjs';
```

Or in *CommonJS* path:

```ts
const { createTypstCompiler } = require('@myriaddreamin/typst.ts/dist/cjs/compiler.cjs');
```

== Add or remove source/binary files

You can also use the `{map,unmap,reset}Shadow` function to manipulate any text or binary file data for typst compiler. They will shadow the file access from provided access model directly in memory.

The `mapShadow(path: string, content: Uint8Array): void;` resembles `addSource(path: string, source: string): void;`, but retrieves some binary data without guessing the underlying encoding.

Example usage:

```ts
const encoder = new TextEncoder();
// add a json file (utf8)
compiler.mapShadow('/assets/data.json', encoder.encode(jsonData));
// remove a json file
compiler.unmapShadow('/assets/data.json');
// clean up all shadow files (Note: this function will also clean all files added by `addSource`)
compiler.resetShadow();

// add an image file
const pngData = await fetch(...).arrayBuffer();
compiler.mapShadow('/assets/tiger.png', new Uint8Array(pngData));
```

== Specify output format

Export document as #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/docs/proposals/8-vector-representation-for-rendering.typ")[_Vector Format_] which can then load to the renderer to render the document.

```ts
const artifactData = await compiler.compile({
  mainFilePath: '/main.typ',
  // the default value of format field:
  // format: 'vector',
});
```

== Specify extra initialization options

You must specify the extra init options when calling the `init` function. For example, to load the wasm module from a custom path:

```js
await cc.init({
  getModule: () =>
    '/path/to/typst_ts_web_compiler_bg.wasm',
});
```
