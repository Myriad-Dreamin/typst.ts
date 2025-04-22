#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Get Started")

= Get Started

In this chapter, you will learn the #link(<assets>)[assets] provided by typst.ts, #link(<import>)[import] it to your project, and run the #link(<run-compiler>)[compiler] or #link(<run-renderer>)[renderer] module with typst.ts.

== Assets <assets>

To get functionality of #link("https://typst.app")[typst], typst.ts provides a core JavaScript library along with two Wasm library:
- `typst.ts`: the core JavaScript library which wraps Wasm modules with more friendly JavaScript APIs.
- `typst-ts-renderer`: a Wasm module that provides rendering functionality.
- `typst-ts-web-compiler`: a Wasm module that provides compilation functionality.

You can install them via #link("https://www.npmjs.com/")[npm] or #link("https://yarnpkg.com/")[Yarn] separately (npm as an example):

```bash
npm install @myriaddreamin/typst.ts
# Optional: if you want to run a typst renderer.
npm install @myriaddreamin/typst-ts-renderer
# Optional: if you want to run a typst compiler.
npm install @myriaddreamin/typst-ts-web-compiler
```

== Import typst.ts to your project <import>

#let easy_color = green.darken(25%)
#let hard_color = red.darken(25%)

There are several ways to setup typst.ts. The difficulty of each approach is evaluated by how many resources you need to configure and whether you need to be familiar with #text(fill: easy_color, [JavaScript]) or #text(fill: hard_color, [Rust]).

#let difficult-easy = text(fill: easy_color, "easy")
#let difficult-medium = text(fill: orange.darken(25%), "medium")
#let difficult-hard = text(fill: hard_color, "hard")

- #box(link(<approach-all-in-one-node>)[Approach 1]) (Recommended in Node.js)
  start with the all-in-one Node.js Library.

- #box(link(<approach-all-in-one>)[Approach 2]) (Recommended in Browser)
  start with the all-in-one JavaScript Library.

- #box(link(<approach-bundle>)[Approach 3])
  Use a bundled javascript file along with wasm modules.

- #box(link(<approach-node-lib>)[Approach 4])
  Use typst.ts as a library in Node.js.

- #box(link(<approach-ts-lib>)[Approach 5])
  Use typst.ts as a library in browser (for TypeScript users).

- #box(link(<approach-js-lib>)[Approach 6])
  Use typst.ts as a library in browser (for JavaScript users).

- #box(link(<approach-ts-lib-from-source>)[Approach 7])
  Use typst.ts with customized renderer/compiler modules.

#line(length: 100%)

=== Simple compiler and renderer bindings to Node.js <approach-all-in-one-node>
#let easy-compiler-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/projects/hexo-renderer-typst/lib/compiler.cjs")[Compiler]
#let easy-renderer-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/projects/hexo-renderer-typst/lib/renderer.cjs")[Renderer]

Difficulty: #difficult-easy, Example: #easy-compiler-example and #easy-renderer-example for #link("https://hexo.io/")[Hexo]

The compiler and renderer are integrated into a same node library for simpler and cleaner APIs, since there is no urgent need to tree-shake the components in node.js applications.

```ts
const compiler = NodeCompiler.create();
await compiler.pdf({
  mainFileContent: 'Hello, typst!',
}); // :-> PDF Buffer
```

See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one-node.html")[All-in-one Node.js Library] for more example usage.

=== Run the compiler or renderer with simplified APIs <approach-all-in-one>
#let easy-preview-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[Single HTML file for real-time previewing typst document]

Difficulty: #difficult-easy, Example: #easy-preview-example

The most simple examples always work with the all-in-one JavaScript Library:

```ts
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
console.log((await $typst.svg({
  mainContent: 'Hello, typst!' })).length);
// :-> 7317
```

See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one.html")[All-in-one (Simplified) JavaScript Library] for more example usage.

Once you feel more conformtable, please continue to try other approaches.

=== Use a bundled javascript file along with wasm modules. <approach-bundle>
#let bundle-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[Single HTML file]

Difficulty: #difficult-easy, Example: #bundle-example

You can include a single bundle file of `@myriaddreamin/typst.ts` in your html file and load needed wasm modules via `fetch` api.

```html
<script type="module"
  src="/core/dist/esm/main.bundle.js"></script>
<script>
let renderModule = window.TypstRenderModule;
let renderPlugin =
  renderModule.createTypstRenderer();
renderPlugin
  .init({
   getModule: () => fetch(
    'path/to/typst_ts_renderer_bg.wasm'),
  })
  .then(async () => {
    console.log('renderer initialized', renderPlugin);
    // do something with renderPlugin
  });
</script>
```

See #bundle-example for a complete example.

=== Use typst.ts as a library in Node.js. <approach-node-lib>

Difficulty: #difficult-easy

You can import typst.ts as a library:

```typescript
import { createTypstRenderer } from
  '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';

const renderer = createTypstRenderer();
renderer.init({}).then(...);
```

There are several templates for developing typst.ts with Node.js:

- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/node.js")[Use renderer, with typescript configured with:]
  ```json { "moduleResolution": "Node" }``` or #linebreak()
  ```json { "moduleResolution": "Node10" }```
- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/node.js-next")[Use renderer, with typescript configured with:]
  ```json { "moduleResolution": "Node16" }``` or #linebreak()
  ```json { "moduleResolution": "NodeNext" }```
- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/ts-node")[Use ts-node, with typescript configured with:]
  ```json { "moduleResolution": "Node" }``` or #linebreak()
  ```json { "moduleResolution": "Node10" }```
- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/ts-node-next")[Use ts-node, with and typescript configured with:]
  ```json { "moduleResolution": "Node16" }``` or #linebreak()
  ```json { "moduleResolution": "NodeNext" }```
- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/compiler-wasm")[Use compiler in browser, with typescript configured with:]
  ```json { "moduleResolution": "Node16" }``` or #linebreak()
  ```json { "moduleResolution": "NodeNext" }```
- #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/compiler-node")[Use compiler in node.js, with typescript configured with:]
  ```json { "moduleResolution": "Node16" }``` or #linebreak()
  ```json { "moduleResolution": "NodeNext" }```

=== Use typst.ts as a library in browser (for TypeScript users). <approach-ts-lib>

Difficulty: #difficult-medium

You can import typst.ts as a library:

```typescript
import { createTypstRenderer } from
  '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';

const renderer = createTypstRenderer();
renderer.init({
   getModule: () => fetch(...),
  }).then(...);
```

=== Use typst.ts as a library in browser (for JavaScript users). <approach-js-lib>

Difficulty: #difficult-medium

Please ensure your main file is with `mjs` extension so that nodejs can recognize it as an es module.

```shell
node main.mjs
```

=== Use typst.ts with customized renderer/compiler modules. <approach-ts-lib-from-source>

Difficulty: #difficult-hard

People familiar with rust can develop owned wasm modules with typst.ts so that they can eliminate unnecessary features and reduce the size of the final bundle. For example, if you want to build a renderer module that only supports rendering svg, you can build it like this:

```shell
wasm-pack build --target web --scope myriaddreamin -- --no-default-features --features render_svg
```

#line(length: 100%)

=== Configure path to wasm module

You may have modified the path to wasm module or rebuilt the wasm module for your own purpose. In this case, you need to configure the path to wasm module. There is a `getModule` option in `init` function that you can use to configure the path to wasm module:

```ts
renderer.init({
  getModule: () => __wasm_module_resource__,
}).then(...);
```

You can load `__wasm_module_resource__` in several ways:

```ts
// from url
const getModule = () => 'http://...';
// from http request
const getModule = () => fetch('http://...');
// from local file
const { readFileSync } = require('fs');
const getModule = () => new Uint8Array(readFileSync('path/to/wasm/module').buffer);
// instantiated wasm module
const getModule = () => WebAssembly.instantiate(/* params */);
// asynchronously
const getModule = async () => {/* above four ways */};
```

== Configure and run compiler <run-compiler>

- Configure font resources

- Configure access model

- Configure package registry

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/options.init.mts")[options.init.mts] for more details.

=== Precompile with `typst-ts-cli`

See #term.ts-cli for more details.

=== Build a compilation service in rust

See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/service.html")[Compiler Service Library] for more details.

== Configure and run renderer <run-renderer>

- Configure font resources, same as compiler.

See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/options.init.mts")[options.init.mts] for more details.

== Further reading

+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one-node.html")[All-in-one Node.js Library]
+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one.html")[All-in-one (Simplified) JavaScript Library]
+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compilers.html")[Compilers]
+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/renderers.html")[Renderers]
+ #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/trouble-shooting.html")[Trouble shooting]
