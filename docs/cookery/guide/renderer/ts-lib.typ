#import "/docs/cookery/book.typ": *
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "JavaScript/TypeScript Library")

#let renderer-source = "https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/src/renderer.mts"
#let renderer-lib = link(renderer-source)[`renderer.mts`]

= JavaScript/TypeScript Library

Use #link("https://www.npmjs.com/package/@myriaddreamin/typst.ts")[`@myriaddreamin/typst.ts`].

It is also runnable in node.js, but we recommend using the #cross-link("/guide/all-in-one-node.typ")[Node.js Library] whenever. Ideally, we can wrap node.js into `@myriaddreamin/typst.ts`, but we still don't have concrete idea. Please let us know if you're interested.

== Use simplified APIs

One may use simplified apis:

```typescript
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
const mainContent = 'Hello, typst!';

console.log(await $typst.svg({ mainContent }));
```

Specify correct path to the wasm modules if it complains.

```typescript
$typst.setCompilerInitOptions({ getModule: ... });
$typst.setRendererInitOptions({ getModule: ... });
```

The path option is likely required in browser but not in node.js.

See #link("https://myriad-dreamin.github.io/typst.ts/cookery/guide/all-in-one.html")[All-in-one (Simplified) JavaScript Library] for more details.

== Use one-shot APIs

See #renderer-lib for more details.

=== Example: render a precompiled document inside of some `<div/>` element

Full example: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[single-file]

First, initialize the renderer inside browser:

```js
let m = window.TypstRenderModule;
let plugin = m.createTypstRenderer();
plugin
  .init({
    getModule: () =>
      '/path/to/typst_ts_renderer_bg.wasm',
    })
```

Please ensure that `/path/to/typst_ts_renderer_bg.wasm` is accessible to your frontend.

Next, load the artifact in #term.vector-format from somewhere. For example, load it by the `fetch` api:

```ts
const artifactContent: Uint8Array =
  await fetch('/main.white.artifact.sir.in')
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));
```

#include "get-artifact.typ"

Finally, call the `render` api to trigger rendering:

```js
await plugin.init(args);
const artifactContent = await loadData(args);

// <div id="typst-app" />
const container = document.getElementById('typst-app');

await plugin.renderToCanvas({
  artifactContent,
  container,
  backgroundColor: '#343541',
  pixelPerPt: 4.5,
});
```

See the sample application #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/packages/typst.ts/index.html")[single-file] for more details.

== Use `RenderSession` APIs

Full exmaple: #link("https://github.com/Enter-tainer/typst-preview/tree/110c031d21e74f747f78fbf78934140d23fec267/addons/frontend")[typst-preview-frontend]

See #renderer-lib for more details.
