#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "Renderer in Vue3")

= Vue3 Library

Use #link("https://www.npmjs.com/package/@myriaddreamin/typst.vue3")[`@myriaddreamin/typst.vue3`] with `@myriaddreamin/typst.ts`.

```bash
npm install @myriaddreamin/typst.ts @myriaddreamin/typst.vue3
```

Register and render the component in your Vue application:

```vue
<template>
  <Typst v-bind:content="sourceCode" />
</template>
```

When the browser cannot discover the Wasm module files automatically, configure them before the first render:

```ts
import { $typst } from '@myriaddreamin/typst.ts';

$typst.setCompilerInitOptions({
  getModule: () =>
    '/path/to/typst_ts_web_compiler_bg.wasm',
});

$typst.setRendererInitOptions({
  getModule: () =>
    '/path/to/typst_ts_renderer_bg.wasm',
});
```

For production applications, serve the Wasm modules and required fonts from your own asset pipeline instead of relying on development-server paths.
