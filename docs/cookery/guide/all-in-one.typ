#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "All-in-one (Simplified) JavaScript Library")

= All-in-one (Simplified) JavaScript Library

#let snippet-source = "https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/contrib/snippet.mts"
#let snippet-lib = link(snippet-source)[`snippet`]

The most simple examples always work with #snippet-lib utility library, an all-in-one JavaScript Library with simplified API interfaces:

```ts
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

However, it is less flexible and stable than the underlying interfaces, the `TypstCompiler` and `TypstRenderer`. If you've become more familar with typst.ts, we recommend you rewrite your library with underlying interfaces according to example usage shown by the #snippet-lib library.

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

Note: if you want to compile multiple documents, you should create a new instance for each compilation work or maintain the shared state on the utility instance `$typst` carefully,
because the compilation process may change the state of that.

=== Example: Create an instance of the utility class:

```typescript
// optional renderer instance
const renderer = enableRendering ?? (() => {
  return createGlobalRenderer(createTypstRenderer,
    undefined /* pdfJsLib */, initOptions);
});
const $typst = new TypstSnippet(() => {
  return createGlobalCompiler(createTypstCompiler,
    initOptions);
}, renderer);
```

=== Example: get output from input

get output with *single input file*:

```ts
const mainContent = 'Hello, typst!';
// into vector format
await $typst.vector({ mainContent });
// into svg format
await $typst.svg({ mainContent });
// into pdf format
await $typst.pdf({ mainContent });
// into canvas operations
await $typst.canvas(div, { mainContent });
```

get output with *multiple input files*:

```ts
// the default value of main path is '/main.typ'
await $typst.addSource('/main.typ', mainContent);

// set path to main file
const mainFilePath = '/tasks/1/main.typ';
await $typst.setMainFilePath(mainFilePath)
await $typst.addSource(mainFilePath, mainContent);
```

What is quite important is that, when you are running multiple tasks asynchronously or in parallel, the call pattern `await $typst.xxx({ mainContent });` is unsafe (introduces undefined behavior). Insteadly you should call compilation by specifying path to the main file:

```ts
const mainFilePath = '/tasks/1/main.typ';
await $typst.addSource(mainFilePath, mainContent);

// compile source of path
await $typst.svg({ mainFilePath });
```

get output with *binary input files*:

```ts
const encoder = new TextEncoder();
// add a json file (utf8)
compiler.mapShadow('/assets/data.json', encoder.encode(jsonData));
// remove a json file
compiler.unmapShadow('/assets/data.json');

// add an image file
const pngData = await fetch(...).arrayBuffer();
compiler.mapShadow('/assets/tiger.png', new Uint8Array(pngData));
```

clean up shadow files for underlying access model:

```ts
compiler.resetShadow();
```

Note: this function will also clean all files added by `addSource`.

=== Example: reuse compilation result

The compilation result could be stored in an artifact in #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/docs/proposals/8-vector-representation-for-rendering.typ")[_Vector Format_], so that you could decouple compilation from rendering or make high-level cache compilation.

```ts
const vectorData = await $typst.vector({ mainContent });
// into svg format
await $typst.svg({ vectorData });
// into canvas operations
await $typst.canvas(div, { vectorData });
```

Note: the compilation is already cached by typst's `comemo` implicitly.

== Specify extra init options

The extra init options must be at the start of the main routine, or accurately before all invocations.

```ts
// Example: cache default fonts to file system
$typst.setCompilerInitOptions(await cachedFontInitOptoins());
// specify init options to renderer
$typst.setRendererInitOptions(rendererInitOptions);
// wire other `pdfJsLib` instance for renderer
$typst.setPdfjsModule(pdfJsLib);

// The compiler instance is initialized in this call.
await $typst.svg({ mainContent });
```

== Specify extra render options

See #link(snippet-source)[comments on source] for more details.
