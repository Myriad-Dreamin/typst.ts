#import "/docs/cookery/book.typ": book-page

#show: book-page.with(title: "Get Started")

= Get Started

To get functionality of #link("https://typst.app")[typst], typst.ts provides a core JavaScript library along with two Wasm library:
- `@myriaddreamin/typst.ts`: the core JavaScript library which wraps Wasm modules with more friendly JavaScript APIs.
- `@myriaddreamin/typst-ts-renderer`: a Wasm module that provides rendering functionality.
- `@myriaddreamin/typst-ts-web-compiler`: a Wasm module that provides compilation functionality.

You can install them via #link("https://www.npmjs.com/")[npm] or #link("https://yarnpkg.com/")[Yarn] separately (npm as an example):

```bash
npm install @myriaddreamin/typst.ts
# Optional: if you want to run a typst renderer.
npm install @myriaddreamin/typst-ts-renderer
# Optional: if you want to run a typst compiler.
npm install @myriaddreamin/typst-ts-web-compiler
```

There are several ways to play with typst.ts.

=== Use a bundled javascript file along with wasm modules.
#let bundle-example = link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[Single HTML file]

Difficulty: #text(fill: green.darken(25%), "easy"), Example: #bundle-example

You can include a single bundle file of `@myriaddreamin/typst.ts` in your html file and load needed wasm modules via `fetch` api.

```html
<script type="module"
  src="/core/dist/esm/main.bundle.js"></script>
<script>
let renderModule = window.TypstRenderModule;
let renderPlugin = 
  renderModule.createTypstRenderer(pdfjsLib);
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

=== Use typst.ts as a library (for TypeScript users).

Difficulty: #text(fill: orange.darken(25%), "medium")

To use typst.ts as a library, you need to import it in flavor of Es Module. Please ensure that your tsconfig.json is correct:

```json
{
  "compilerOptions": {
      "module": "ESNext",
  }
}
```

Then you can import typst.ts as a library:

```typescript
import { createTypstRenderer } from
  '@myriaddreamin/typst.ts/dist/esm/renderer';

const renderer = createTypstRenderer();
renderer.init({
   getModule: () => fetch(...),
  }).then(...);
```

=== Use typst.ts as a library (for JavaScript users).

Difficulty: #text(fill: orange.darken(25%), "medium")

Please ensure your main file is with `mjs` extension so that nodejs can recognize it as an es module.

```shell
node main.mjs
```

=== Use typst.ts with customized renderer/compiler modules

Difficulty: #text(fill: red.darken(25%), "hard")

People familiar with rust can develop owned wasm modules with typst.ts so that they can eliminate unnecessary features and reduce the size of the final bundle. For example, if you want to build a renderer module that only supports rendering svg, you can build it like this:

```shell
wasm-pack build --target web --scope myriaddreamin -- --no-default-features --features render_svg
```
