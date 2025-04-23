#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-One Library for Browsers")

#include "claim.typ"

Note: This is suitable for running in browser, but not very fit in node.js applications. This is because:
- The compiler for browsers is in wasm module and slower than running compiler as native code.
- You must carefully maintain the bundle size of your browser applications, there for the components are split for better tree-shaking.
- The default fonts to load in browser are for network.

In other words:
- The node.js library runs compiler as native code, thus native performance.
- The compiler and renderer are integrated into a same node library for simpler and cleaner APIs.
- You can simply use system fonts lazily with the compiler for node but not that for web.

If you want to run the compiler or renderer in Node.js, please see #cross-link("/guide/all-in-one-node.typ")[All-in-one Library for Node.js].

#let snippet-source = "https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/contrib/snippet.mts"
#let snippet-lib = link(snippet-source)[`snippet.mts`]

#let sub = heading-reference[== (Archived) The All-in-One Js Library in v0.5.0]
 *Note: the following content is for typst.ts >=v0.6.0. To use rust library in \<v0.6.0, check #cross-link("/guide/all-in-one.typ", reference: sub)[the section.]*

The most simple examples always work with #snippet-lib utility library, an all-in-one library with simplified API interfaces:

```ts
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

However, it is less flexible and stable than the underlying interfaces, the `TypstCompiler` and `TypstRenderer`. If you've become more familiar with typst.ts, we recommend you rewrite your library with underlying interfaces according to example usage shown by the #snippet-lib library.

== Installation

We provide two bundles for the all-in-one library:
- `all-in-one.bundle.js`, it bundles all of the resources to run a typst compiler. You can download the single bundle file and run the compiler offline.
- `all-in-one-lite.bundle.js`, the fonts and Wasm modules are excluded to reduce the bundle size. This will cause the script *load extra resources from CDN*.

Both the `all-in-one.bundle.js` and `all-in-one.bundle.js` can be used without configuration:

```html
<script
  type="module"
  <!-- Loading the full bundle or the lite version from jsdelivr -->
  <!-- src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one.bundle.js" -->
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one-lite.bundle.js"
  id="typst"
>
  console.log($typst.svg({
    mainContent: 'Hello, typst!',
  }));
</script>
```

See a #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[simple and heavily-documented previewer] to learn a more practical example of usage, which watches user input and compiles the content into SVG for preview.

#if not is-pdf-target() {
  raw(block: true, lang: "html", read("/github-pages/preview.html"))
}

== Using a snippet instance

As a shortcut, A global instance `$typst` is provided, and it will lazily initialize the compiler and renderer for you. Import the global instance like this:

```ts
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';
```

A snippet instance will store some state for the sake of convenience, which makes it not suitable for concurrent usage. You can create a new instance using the class `TypstSnippet`:

```typescript
import { TypstSnippet } from '@myriaddreamin/typst.ts/contrib/snippet';
const $typst = new TypstSnippet({
  // optional renderer instance
  renderer: enableRendering ?? (() => {
    return createGlobalRenderer(
      createTypstRenderer, initOptions);
  }),
  compiler() => {
    return createGlobalCompiler(createTypstCompiler,
      initOptions);
  }
});
```

#include "all-in-one-inputs.typ"

=== Spliting compilation and rendering

The compilation result could be stored in an artifact in #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/docs/proposals/8-vector-representation-for-rendering.typ")[_Vector Format_], so that you can decouple compilation from rendering.


You can either compile the document to a vector format in browsers:

```ts
const vectorData = await $typst.vector({ mainContent });
```

or load the data from remote machine, which is precompiled by some tools:

```ts
const remoteData = await (fetch(
    './main.sir.in').then(resp => resp.arrayBuffer()));
const vectorData = new Uint8Array(remoteData);
```

With `vectorData`, you can export it to different formats without compiling again:

```ts
// into svg format
await $typst.svg({ vectorData });
// into canvas operations
await $typst.canvas(div, { vectorData });
```

== Initializing using the low-level API

The extra initialization options must be at the start of the main routine, or accurately before all invocations. You can set the options by low-level or high-level APIs. The low-level APIs are direct but not easy to use:

```ts
// Example: cache default fonts to file system
$typst.setCompilerInitOptions(await cachedFontInitOptions());
// specify init options to renderer
$typst.setRendererInitOptions(rendererInitOptions);

// The compiler instance is initialized in this call.
await $typst.svg({ mainContent });
```

Some examples are still easy. For instance, a commonly usage is configuring the way to obtain the wasm module files:

```js
$typst.setCompilerInitOptions({
  getModule: () =>
    '/path/to/typst_ts_web_compiler_bg.wasm',
});
$typst.setRendererInitOptions({
  getModule: () =>
    '/path/to/typst_ts_renderer_bg.wasm',
});
```

Please check the low-level components to get full reference about these options:
- #cross-link("/guide/compiler/bindings.typ")[Compiler in Wasm (Web)]
- #cross-link("/guide/renderer/ts-lib.typ")[Renderer in Wasm (Web)]

// Note: There are more documentation about initialization in the *Import typst.ts to your project* section of #link("https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html")[Get started with Typst.ts].

== Initializing using the high-level `use` API

todo: fully document the `use` API.

Specify address to a http server for filesystem backend (shadowed by the `addSource` and `mapShadow` api):

```js
const cm = window.TypstCompileModule;
const fetchBackend = new cm.FetchAccessModel(
  'http://localhost:20810',
);
$typst.use(
  TypstSnippet.withAccessModel(fetchBackend),
);
```

Specify a memory filesystem backend (shadowed by the `addSource` and `mapShadow` api):

```js
const memoryAccessModel = new cm.MemoryAccessModel();
$typst.use(
  TypstSnippet.withAccessModel(memoryAccessModel),
);
```

Fetch package from remote registry:

```js
const accessModel = cm.FetchAccessModel() or
  cm.MemoryAccessModel() or others;
$typst.use(
  TypstSnippet.fetchPackageRegistry(fetchBackend),
);
```

// == Specify extra render options

// See #link(snippet-source)[comments on source] for more details.

// === Sample application: real-time preview document

== (Archived) The All-in-One Js Library in v0.5.0

The most simple examples always work with #snippet-lib utility library, an all-in-one library with simplified API interfaces:

```ts
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

However, it is less flexible and stable than the underlying interfaces, the `TypstCompiler` and `TypstRenderer`. If you've become more familiar with typst.ts, we recommend you rewrite your library with underlying interfaces according to example usage shown by the #snippet-lib library.

Note: If your script targets to *CommonJS*, you should import it in *CommonJS* path instead of In *ES Module* path:

```ts
const { createTypstCompiler } = require(
  '@myriaddreamin/typst.ts/dist/cjs/compiler.cjs');
```

== Examples

Here are some examples for the #snippet-lib utility library.

=== Example: Use the _global shared_ compiler instance:

```typescript
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
```

=== Example: Create an instance of the utility class:

```typescript
const $typst = new TypstSnippet({
  // optional renderer instance
  renderer: enableRendering ?? (() => {
    return createGlobalRenderer(
      createTypstRenderer, initOptions);
  }),
  compiler() => {
    return createGlobalCompiler(createTypstCompiler,
      initOptions);
  }
});
```

#include "all-in-one-inputs.typ"

=== Example: reuse compilation result

The compilation result could be stored in an artifact in #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/docs/proposals/8-vector-representation-for-rendering.typ")[_Vector Format_], so that you could decouple compilation from rendering or make high-level cache compilation.

```ts
const vectorData = await $typst.vector({ mainContent });
// or load vector data from remote
const remoteData = await (fetch(
    './main.sir.in').then(resp => resp.arrayBuffer()));
const vectorData = new Uint8Array(remoteData);

// into svg format
await $typst.svg({ vectorData });
// into canvas operations
await $typst.canvas(div, { vectorData });
```

Note: the compilation is already cached by typst's `comemo` implicitly.

== Specify extra init options

Ideally, you don't have to specify any options. But if necessary, the extra init options must be at the start of the main routine, or accurately before all invocations.

```ts
// Example: cache default fonts to file system
$typst.setCompilerInitOptions(await cachedFontInitOptions());
// specify init options to renderer
$typst.setRendererInitOptions(rendererInitOptions);

// The compiler instance is initialized in this call.
await $typst.svg({ mainContent });
```

Note: There are more documentation about initialization in the *Import typst.ts to your project* section of #link("https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html")[Get started with Typst.ts].

== Configure snippet by the `use` API

Specify address to a http server for filesystem backend (shadowed by the `addSource` and `mapShadow` api):

```js
const cm = window.TypstCompileModule;
const fetchBackend = new cm.FetchAccessModel(
  'http://localhost:20810',
);
$typst.use(
  TypstSnippet.withAccessModel(fetchBackend),
);
```

Specify a memory filesystem backend (shadowed by the `addSource` and `mapShadow` api):

```js
const memoryAccessModel = new cm.MemoryAccessModel();
$typst.use(
  TypstSnippet.withAccessModel(memoryAccessModel),
);
```

Fetch package from remote registry:

```js
const accessModel = cm.FetchAccessModel() or
  cm.MemoryAccessModel() or others;
$typst.use(
  TypstSnippet.fetchPackageRegistry(fetchBackend),
);
```

== Specify extra render options

See #link(snippet-source)[comments on source] for more details.

=== Sample application: real-time preview document

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/all-in-one.html")[Preview by all-in-one Library] by a single included file (`all-in-one.bundle.js`).

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/all-in-one-lite.html")[Preview by all-in-one-lite Library] by the more practical single included file (`all-in-one-lite.bundle.js`), which needs configure your frontend to have access to wasm module files:

```js
$typst.setCompilerInitOptions({
  getModule: () =>
    '/path/to/typst_ts_web_compiler_bg.wasm',
});
$typst.setRendererInitOptions({
  getModule: () =>
    '/path/to/typst_ts_renderer_bg.wasm',
});
```
