#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-One Library for Browsers")

#include "claim.typ"

#let snippet-source = "https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/contrib/snippet.mts"
#let snippet-lib = link(snippet-source)[`snippet.mts`]

#let sub = heading-reference[== (Archived) The All-in-One Js Library in v0.5.0]
*Note: the following content is for typst.ts >=v0.6.0. To use rust library in \<v0.6.0, check #cross-link("/guide/all-in-one.typ", reference: sub)[the section.]*

The all-in-one library provides a simplified API, and you can easily compile typst documents into artifacts. For example, compiling a typst code string to a SVG:

```ts
await $typst.svg({mainContent: 'Hello, typst!' }))
```

However, it is less flexible and stable than the underlying interfaces, the `TypstCompiler` and `TypstRenderer`. If you've become more familiar with typst.ts, we recommend you rewrite your library with underlying interfaces according to example usage. The best ways to use the underlying libraries can be discovered in the source code of the all-in-one library, the #snippet-lib. For example, the above example calls the underlying components:

```ts
$typst.svg(options) <=>
  // Initializes lazily
  compiler = createTypstCompiler(); // TypstCompiler
  await compiler.init(...)
  renderer = createTypstRenderer(); // TypstRenderer
  await renderer.init(...)

  // Adds main file content: 'Hello, typst!'
  await this.addSource(`/tmp/${randstr()}.typ`, options.mainContent);
  // Compiles it
  vectorData = options.vectorData || compiler.compile(options)
  return renderer.runWithSession(session => {
    renderer.manipulateData({ session, action: 'reset', data: vectorData })
    // Renders it
    renderer.renderSvg({ ...options, session })
  })
```

== When to use this library (Wasm v.s. napi)

typst.ts runs official typst compiler and its customized renderers in *Wasm modules*. This is suitable for running in browser, but not very fit in *Node.js* applications. This is because:
- _Slower Speed_: The compiler for browsers is in Wasm module and slower than running compiler as native code.
- _Complex API_: You must carefully maintain the bundle size of your browser applications, Therefore, the components are split for better tree-shaking. This will increase code complexity.
- _Network Access_: It doesn't bundle the fonts, so has to load them from network.

In other words, the #cross-link("/guide/all-in-one-node.typ")[*Node.js* library] achieves:
- _Faster Speed_: The Node.js library runs compiler as native code, thus native performance.
- _Simple API_: The compiler and renderer are integrated into a same Node.js library for simpler and cleaner APIs.
- _Rich Fonts_: The compiler simply use embedded or system fonts for Node.js but not that for web.

If you want to run typst in Node.js using the #link("https://napi.rs/")[napi], please see #cross-link("/guide/all-in-one-node.typ")[All-in-one Library for Node.js].

== Installing Bundles from CDN

We provide two bundles for the all-in-one library:
- `all-in-one.bundle.js`, it bundles all of the resources to run a typst compiler. You can download the single bundle file and run the compiler offline.
- `all-in-one-lite.bundle.js`, the fonts and Wasm modules are excluded to reduce the bundle size. This will cause the script to *load extra resources from CDN*.

Using `all-in-one.bundle.js`:

```html
<script
  type="module"
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-all-in-one.ts@0.6.1-rc3/dist/esm/index.js"
  id="typst"
>
  console.log($typst.svg({
    mainContent: 'Hello, typst!',
  }));
</script>
```

Or `all-in-one-lite.bundle.js` which needs configure your frontend to have access to wasm module files:

```html
<script
  type="module"
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one-lite.bundle.js"
  id="typst"
>
  /// Initializes the Typst compiler and renderer. Since we use "all-in-one-lite.bundle.js" instead of
  /// "all-in-one.bundle.js" we need to tell that the wasm module files can be loaded from CDN (jsdelivr).
  $typst.setCompilerInitOptions({
    getModule: () =>
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
  });
  $typst.setRendererInitOptions({
    getModule: () =>
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
  });

  console.log($typst.svg({
    mainContent: 'Hello, typst!',
  }));
</script>
```

See a #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[simple and heavily-documented previewer] to learn a more practical example of usage, which watches user input and compiles the content into SVG for preview.

#if not is-pdf-target() {
  raw(block: true, lang: "html", read("/github-pages/preview.html"))
}

// === Sample application: real-time preview document

// See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/all-in-one.html")[Preview by all-in-one Library] by a single included file (`all-in-one.bundle.js`).

// See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/examples/all-in-one-lite.html")[Preview by all-in-one-lite Library] by the more practical single included file (`all-in-one-lite.bundle.js`), which needs configure your frontend to have access to wasm module files:

== Installing by Package Managers

You can also install the library from registry, npm as an example:

```bash
npm install @myriaddreamin/typst.ts
# Optional: if you want to run a typst renderer.
npm install @myriaddreamin/typst-ts-renderer
# Optional: if you want to run a typst compiler.
npm install @myriaddreamin/typst-ts-web-compiler
```

Then, you can import the library in your code:

```ts
import { $typst } from '@myriaddreamin/typst.ts';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

#let sub = heading-reference[== Initializing using the low-level API]
In Node.js, it reads and loads the Wasm modules from `node_modules` in the filesystem. If you aim to use the library in browsers, you may need to configure the library to load the wasm module files. Please check #cross-link("/guide/all-in-one.typ", reference: sub)[the section.]

== Using a snippet instance

As a shortcut, a global instance `$typst` is provided, and it will lazily initialize the compiler and renderer for you. Import the global instance like this:

```ts
import { $typst } from '@myriaddreamin/typst.ts';
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

=== Splitting compilation and rendering

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

In this way, the library only requires the `typst-ts-renderer` module for rendering.

== Initializing using the low-level API

The extra initialization options must be at the start of the main routine, or accurately before all invocations. You can set the options by low-level or high-level APIs. The low-level APIs are direct but not easy to use:

```ts
// Specifies init options of renderer
$typst.setRendererInitOptions(rendererInitOptions);
// Specifies init options of compiler. For example, cache default fonts to file system
$typst.setCompilerInitOptions(await cachedFontInitOptions());

// The compiler instance is initialized in this call. After that, setting options makes no sense.
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

== Initializing using the high-level `use` API

todo: fully document the `use` API.
- `preloadFontFromUrl`
- `preloadFontData`
- `preloadFonts`
- `disableDefaultFontAssets`
- `loadFonts`
- `preloadFontAssets`
- `withPackageRegistry`
- `withAccessModel`
- `fetchPackageRegistry`
- `fetchPackageBy`

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
