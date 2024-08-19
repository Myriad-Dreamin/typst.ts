#import "mod.typ": *

#show: book-page.with(title: "Serverless rendering")

#include "claim.typ"

Example Application: #link("https://github.com/Myriad-Dreamin/typst.ts/blob/main/github-pages/preview.html")[single-file]

Run the entire typst directly in browser, like #link("https://typst.app")[typst.app].

== Key Descriptions

It uses #cross-link("/guide/all-in-one.typ")[All-in-one (Simplified) Library for Browsers], using a single bundle for both compiler and renderer. Thought it is less flexible and stable than the underlying interfaces, it is pretty easy. The simplified library also teaches you how to use the underlying interfaces for better performance.

```js
<script
  type="module"
  src="https://cdn.jsdelivr.net/npm/@myriaddreamin/typst.ts/dist/esm/contrib/all-in-one-lite.bundle.js"
  id="typst"
></script>
```

A ```html <textarea>``` is used to input the typst document, and a ```html <div>``` is used to render the document.

```js
<textarea id="input">Hello, Typst!</textarea>
<div id="content"></div>
```

To load wasm modules compiled from Rust from CDN, you need to set the module path for the compiler and renderer:

```js
$typst.setCompilerInitOptions({
  getModule: () =>
    'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
});
$typst.setRendererInitOptions({
  getModule: () =>
    'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
});
```

Sets input event handler and performs a first-time rendering:

```js
input.oninput = () => {
  previewSvg(input.value);
  input.style.height = '5px';
  input.style.height = input.scrollHeight + 'px';
};
previewSvg(input.value);
```
