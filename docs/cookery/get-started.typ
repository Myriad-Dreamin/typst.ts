#import "mod.typ": *
#import "/docs/cookery/book.typ": book-page, cross-link, heading-reference
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Get Started")

= Get Started

// In this chapter, you will learn the #link(<assets-and-libraries>)[core libraries and assets] provided by typst.ts, a #link(<starter-example>)[starter example], and #link(<practical-example>)[practical examples].

#let this-link = cross-link.with("/get-started.typ")

#let ch-ref = heading-reference[== Core Libraries and Assets]
#let h2 = heading-reference[== A starter example]
#let h3 = heading-reference[== Practical Examples]

In this chapter, you will learn the #this-link(reference: ch-ref)[core libraries and assets] provided by typst.ts, a #this-link(reference: h2)[starter example], and #this-link(reference: h3)[practical examples].

== Core Libraries and Assets <assets-and-libraries>

#let size-data = json("/assets/data/bundle-size.json");

The functionalities of #link("https://typst.app")[typst] is split into two parts, _compilation and rendering_ functionality, because no all applications need both functionalities running in the browsers. It will take 350 KB network bandwidth if you want to use the renderer in browser, but it will take 12MB (7.62 MB wasm and 4.42 MB fonts) to run a compiler. Therefore, the two parts are separated into two Wasm modules, `typst-ts-renderer` and `typst-ts-web-compiler`, and they can be loaded on demand.

#let size-header = table.cell(align: center)[Size (gzipped)]

#let paint-size(d) = {
  let human = if (d < 1024 * 1024) {
    [#calc.round(d / 1024, digits: 2) KB]
  } else {
    [#calc.round(d / 1024 / 1024, digits: 2) MB]
  }

  let max-size = calc.max(..size-data.sizes.values())
  let ratio = d / max-size

  // It is hard to change alignment in HTML. Therefore, we prefer different layouts for different targets.
  if is-html-target {
    html.elem("data", attrs: (value: str(d)), human)
    // Here we use `<span>` instead of `<div>`, or typst will create a redundant `<p>`.
    html.elem("span", attrs: (
      aria-hidden: "true",
      style: ```
        display: block;
        background: var(--theme-popup-border);
        width: [[width]];
        height: 0.2em;
      ```
        .text
        .replace("[[width]]", repr(ratio * 100%)),
    ))
  } else {
    context {
      let header-width = measure(size-header).width

      place(horizon + start, box(
        fill: blue.lighten(80%),
        width: ratio * header-width * 95%,
        height: 0.8em,
      ))
    }
    human
  }
}

#align(center, table(
  align: (center, end, center),
  columns: 3,
  [Assets], size-header, [Description],
  `typst-ts-renderer`, paint-size(size-data.sizes.typst-ts-renderer), [For _rendering_],
  `typst-ts-web-compiler`, paint-size(size-data.sizes.typst-ts-web-compiler), [For _compiling_],
  `Text+Math+Raw Fonts`, paint-size(size-data.sizes.text-math-fonts), [To typeset text],
  `CJK Fonts`, paint-size(size-data.sizes.cjk-fonts), [To typeset CJK text],
  `Emoji Fonts`, paint-size(size-data.sizes.emoji-fonts), [To typeset emojis],
))

typst.ts provides core JavaScript libraries along with two Wasm modules:
- `typst.ts`: the core JavaScript library which wraps Wasm modules with more friendly JavaScript APIs.
- `typst-ts-renderer`: a Wasm module providing _rendering_ functionality.
- `typst-ts-web-compiler`: a Wasm module providing _compilation_ functionality.
- `typst-ts-node-compiler`: a Node-native library providing both _compilation and rendering_ functionality.

The server-side rendering is also supported by following packages:
- `typst-ts-cli`: A command line tool providing _compilation_ functionality.
- `reflexo-typst`: A Rust library providing _compilation_ functionality.
- `typst.ts` + `typst-ts-renderer`: The JavaScript library mentioned above providing _rendering_ functionality.

There is also a vite integration providing both _compilation and rendering_ functionality. It is experimental and has not been finished, but I put it here to let you know.

- `vite-plugin-typst`: A vite plugin providing both _compilation and rendering_ functionality.

=== Web Integration

You can install them via #link("https://www.npmjs.com/")[npm] or #link("https://yarnpkg.com/")[Yarn] separately (npm as an example):

```bash
npm install @myriaddreamin/typst.ts
# Optional: if you want to run a typst renderer.
npm install @myriaddreamin/typst-ts-renderer
# Optional: if you want to run a typst compiler.
npm install @myriaddreamin/typst-ts-web-compiler
```

=== Node.js Integration

The compiler and renderer are integrated into a same node library for simpler and cleaner APIs, because there is no urgent need to tree-shake the components in node.js applications.

```bash
npm install @myriaddreamin/typst-ts-node-compiler
```

== A starter example <starter-example>

To simplify the build up, we provide _all-in-one libraries_ for both web and node.js integration. We show the way to render typst documents into a single SVG in the browser and node.js.

=== The starter example in Web

To get it in browser, you can load a all-in-one bundle script from CDN and use it directly:

```html
<script
  type="module"
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-all-in-one.ts@0.7.0-rc2/dist/esm/index.js"
  id="typst"
>
  console.log($typst.svg({
    mainContent: 'Hello, typst!',
  }));
</script>
```

There is also a #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[simple and heavily-documented single-file typst previewer] that compiles and renders a typst document on editing.

*Warning: the all-in-one.bundle.js is not practical to use since it bundles all of the resource regardless you need or not, including the wasm modules and scripts.*

=== The starter example in Node.js

It is even simpler in Node.js, you can use the all-in-one node library practically:

```js
import { $typst } from '@myriaddreamin/typst-ts-node-compiler';
const compiler = NodeCompiler.create();
console.log(await compiler.svg({
  mainFileContent: 'Hello, typst!',
})); // :-> 7317
```

== Practical Examples <practical-example>

Given the #this-link(reference: ch-ref)[Core Libraries and Assets], we can start to build typst applications for web.

Rust Tools:
- #link("https://github.com/Myriad-Dreamin/shiroa")[shiroa]: It is a Rust tool using `reflexo-typst`, producing static HTML files or dynamic ones utilizing `typst.ts` + `typst-ts-renderer`.
  - This documentation is built using shiroa, the #link("https://myriad-dreamin.github.io/typst.ts/cookery/")[HTML version (browser typesetting)] of the documentation.
  - the #link("https://myriad-dreamin.github.io/typst.ts/cookery/paged")[Paged version (typst typsetting)] of the documentation.
- #link("https://github.com/Myriad-Dreamin/tinymist/tree/main/crates/typst-preview")[typst-preview]: It is a Rust tool using `reflexo-typst`, incrementally rendering typst documents to provide preview editing. The data is streamed and rendered in browser using `typst.ts` + `typst-ts-renderer`.

JavaScript Tools:
- The starter example, #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[a simple typst previewer], is already good. It is a single-file HTML using `typst-ts-web-compiler`, producing ```html <svg/>``` and ```html <canvas/>``` for preview utilizing `typst.ts` + `typst-ts-renderer`.
- The #link("https://mitex-rs.github.io/mitex/tools/underleaf.html")[underleaf] is a web-based tex editor. It uses same components as the starter example, but arranges source code in a more complex way. It renders tex files by running a #link("https://github.com/mitex-rs/mitex")[simple tex converter] in typst compiler.
- The #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/rehype-typst")[rehype-typst] compiles typst equations in markdown files using `typst-ts-node-compiler`.
- The #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/hexo-renderer-typst")[hexo-renderer-typst] renders typst documents as web pages for the blog engine, Hexo, using `typst-ts-node-compiler`.
- The #link("https://github.com/typst-doc-cn/news/tree/main/scripts")[Typst CN News] watches and translates typst documents into HTML files by Node.js scripts, using `typst-ts-node-compiler`.
- The #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/projects/vite-plugin-typst")[vite-plugin-typst] watches and translates typst documents into HTML files for vite, using `typst-ts-node-compiler`.

Wrappers of `typst.ts` + `typst-ts-renderer` to integrate into frameworks, which can load artifacts of compilers, are also good examples to study.

First, compiles the artifacts like the rust and javascript tools above:
- `typst-ts-cli` to compile in terminal or bash scripts.
- `reflexo-typst` to compile in Rust applications.
- `vite-plugin-typst` to compile and process with vite (this hasn't been published yet).
- `typst-ts-node-compiler` to compile in Node.js scripts.
- `typst.ts` + `typst-ts-web-compiler` to compile in browsers.

Then, renders the artifacts using the wrappers:
- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[`@myriaddreamin/typst.react`:]

  ```tsx
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

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.angular")[`@myriaddreamin/typst.angular`:]

  In the module file of your awesome component.

  ```typescript
  /// component.module.ts
  import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
  ```

  Using directive `typst-document` in your template file.

  ```html
  <typst-document fill="#343541" artifact="{{ artifact }}"></typst-document>
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.vue3")[`@myriaddreamin/typst.vue3`:]

  ```vue
  <template>
    <Typst v-bind:content="sourceCode" />
  </template>
  ```

- Using #link("https://www.npmjs.com/package/@myriaddreamin/typst.solid")[`@myriaddreamin/typst.solid`:]

  ```tsx
  import { TypstDocument } from '@myriaddreamin/typst.solid';

  export const App = (artifact: Uint8Array) => {
    return (
      <div>
        <h1>Demo: Embed Your Typst Document in Solid </h1>
        <TypstDocument fill="#343541" artifact={vec()} />
      </div>
    );
  };
  ```

// == Import typst.ts to your project <import>

// #let easy_color = green.darken(25%)
// #let hard_color = red.darken(25%)

// There are several ways to setup typst.ts. The difficulty of each approach is evaluated by how many resources you need to configure and whether you need to be familiar with #text(fill: easy_color, [JavaScript]) or #text(fill: hard_color, [Rust]).

// #let difficult-easy = text(fill: easy_color, "easy")
// #let difficult-medium = text(fill: orange.darken(25%), "medium")
// #let difficult-hard = text(fill: hard_color, "hard")

// - #box(link(<approach-all-in-one-node>)[Approach 1]) (Recommended in Node.js)
//   start with the all-in-one Node.js Library.

// - #box(link(<approach-all-in-one>)[Approach 2]) (Recommended in Browser)
//   start with the all-in-one JavaScript Library.

// - #box(link(<approach-bundle>)[Approach 3])
//   Use a bundled javascript file along with wasm modules.

// - #box(link(<approach-node-lib>)[Approach 4])
//   Use typst.ts as a library in Node.js.

// - #box(link(<approach-ts-lib>)[Approach 5])
//   Use typst.ts as a library in browser (for TypeScript users).

// - #box(link(<approach-js-lib>)[Approach 6])
//   Use typst.ts as a library in browser (for JavaScript users).

// - #box(link(<approach-ts-lib-from-source>)[Approach 7])
//   Use typst.ts with customized renderer/compiler modules.

// #line(length: 100%)

// === Simple compiler and renderer bindings to Node.js <approach-all-in-one-node>
// #let easy-compiler-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/projects/hexo-renderer-typst/lib/compiler.cjs")[Compiler]
// #let easy-renderer-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/projects/hexo-renderer-typst/lib/renderer.cjs")[Renderer]

// Difficulty: #difficult-easy, Example: #easy-compiler-example and #easy-renderer-example for #link("https://hexo.io/")[Hexo]

// The compiler and renderer are integrated into a same node library for simpler and cleaner APIs, since there is no urgent need to tree-shake the components in node.js applications.

// ```ts
// const compiler = NodeCompiler.create();
// await compiler.pdf({
//   mainFileContent: 'Hello, typst!',
// }); // :-> PDF Buffer
// ```

// See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one-node.html")[All-in-one Node.js Library] for more example usage.

// === Run the compiler or renderer with simplified APIs <approach-all-in-one>
// #let easy-preview-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[Single HTML file for real-time previewing typst document]

// Difficulty: #difficult-easy, Example: #easy-preview-example

// The most simple examples always work with the all-in-one JavaScript Library:

// ```ts
// import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
// console.log((await $typst.svg({
//   mainContent: 'Hello, typst!' })).length);
// // :-> 7317
// ```

// See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one.html")[All-in-one (Simplified) JavaScript Library] for more example usage.

// Once you feel more conformtable, please continue to try other approaches.

// === Use a bundled javascript file along with wasm modules. <approach-bundle>
// #let bundle-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[Single HTML file]

// Difficulty: #difficult-easy, Example: #bundle-example

// You can include a single bundle file of `@myriaddreamin/typst.ts` in your html file and load needed wasm modules via `fetch` api.

// ```html
// <script type="module"
//   src="/core/dist/esm/main.bundle.js"></script>
// <script>
// let renderModule = window.TypstRenderModule;
// let renderPlugin =
//   renderModule.createTypstRenderer();
// renderPlugin
//   .init({
//    getModule: () => fetch(
//     'path/to/typst_ts_renderer_bg.wasm'),
//   })
//   .then(async () => {
//     console.log('renderer initialized', renderPlugin);
//     // do something with renderPlugin
//   });
// </script>
// ```

// See #bundle-example for a complete example.

// === Use typst.ts as a library in Node.js. <approach-node-lib>

// Difficulty: #difficult-easy

// You can import typst.ts as a library:

// ```typescript
// import { createTypstRenderer } from
//   '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';

// const renderer = createTypstRenderer();
// renderer.init({}).then(...);
// ```

// There are several templates for developing typst.ts with Node.js:

// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/node.js")[Use renderer, with typescript configured with:]
//   ```json { "moduleResolution": "Node" }``` or #linebreak()
//   ```json { "moduleResolution": "Node10" }```
// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/node.js-next")[Use renderer, with typescript configured with:]
//   ```json { "moduleResolution": "Node16" }``` or #linebreak()
//   ```json { "moduleResolution": "NodeNext" }```
// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/ts-node")[Use ts-node, with typescript configured with:]
//   ```json { "moduleResolution": "Node" }``` or #linebreak()
//   ```json { "moduleResolution": "Node10" }```
// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/ts-node-next")[Use ts-node, with and typescript configured with:]
//   ```json { "moduleResolution": "Node16" }``` or #linebreak()
//   ```json { "moduleResolution": "NodeNext" }```
// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/compiler-wasm")[Use compiler in browser, with typescript configured with:]
//   ```json { "moduleResolution": "Node16" }``` or #linebreak()
//   ```json { "moduleResolution": "NodeNext" }```
// - #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/templates/compiler-node")[Use compiler in node.js, with typescript configured with:]
//   ```json { "moduleResolution": "Node16" }``` or #linebreak()
//   ```json { "moduleResolution": "NodeNext" }```

// === Use typst.ts as a library in browser (for TypeScript users). <approach-ts-lib>

// Difficulty: #difficult-medium

// You can import typst.ts as a library:

// ```typescript
// import { createTypstRenderer } from
//   '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';

// const renderer = createTypstRenderer();
// renderer.init({
//    getModule: () => fetch(...),
//   }).then(...);
// ```

// === Use typst.ts as a library in browser (for JavaScript users). <approach-js-lib>

// Difficulty: #difficult-medium

// Please ensure your main file is with `mjs` extension so that nodejs can recognize it as an es module.

// ```shell
// node main.mjs
// ```

// === Use typst.ts with customized renderer/compiler modules. <approach-ts-lib-from-source>

// Difficulty: #difficult-hard

// People familiar with rust can develop owned wasm modules with typst.ts so that they can eliminate unnecessary features and reduce the size of the final bundle. For example, if you want to build a renderer module that only supports rendering svg, you can build it like this:

// ```shell
// wasm-pack build --target web --scope myriaddreamin -- --no-default-features --features render_svg
// ```

// #line(length: 100%)

// === Configure path to wasm module

// You may have modified the path to wasm module or rebuilt the wasm module for your own purpose. In this case, you need to configure the path to wasm module. There is a `getModule` option in `init` function that you can use to configure the path to wasm module:

// ```ts
// renderer.init({
//   getModule: () => __wasm_module_resource__,
// }).then(...);
// ```

// You can load `__wasm_module_resource__` in several ways:

// ```ts
// // from url
// const getModule = () => 'http://...';
// // from http request
// const getModule = () => fetch('http://...');
// // from local file
// const { readFileSync } = require('fs');
// const getModule = () => new Uint8Array(readFileSync('path/to/wasm/module').buffer);
// // instantiated wasm module
// const getModule = () => WebAssembly.instantiate(/* params */);
// // asynchronously
// const getModule = async () => {/* above four ways */};
// ```

// == Configure and run compiler <run-compiler>

// - Configure font resources

// - Configure access model

// - Configure package registry

// See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/options.init.mts")[options.init.mts] for more details.

// === Precompile with `typst-ts-cli`

// See #term.ts-cli for more details.

// === Build a compilation service in rust

// See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/compiler/service.html")[Compiler Service Library] for more details.

// == Configure and run renderer <run-renderer>

// - Configure font resources, same as compiler.

// See #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/options.init.mts")[options.init.mts] for more details.

== Further reading

+ #cross-link("/start-it.typ")[All-in-One Libraries]
+ #cross-link("/guide/compilers.typ")[Compilers]
+ #cross-link("/guide/renderers.typ")[Renderers]
+ #cross-link("/direction/main.typ")[Samples]
+ #cross-link("/guide/trouble-shooting.typ")[Trouble shooting]
