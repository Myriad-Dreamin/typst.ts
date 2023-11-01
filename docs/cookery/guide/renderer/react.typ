#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "React Library")

= React Library

Use #link("https://www.npmjs.com/package/@myriaddreamin/typst.react")[`@myriaddreamin/typst.react`].

```typescript
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

=== `fill` property

Fill document with color.

```html
<TypstDocument fill="#343541"/>
```

Note: Current typst.ts doesn't support a transparent background color in some browsers.

=== `artifact` property

Render the document with artifact from precompiler.

```html
<TypstDocument artifact={artifact}/>
```

The artifact can be only in #term.vector-format to this time.

To get `artifact` data, please refer to #term.ts-cli.

#include "pdfjs.typ"

=== Set renderer initialization option for `TypstDocument`

Retrieve a #term.init-option for initializating the renderer for `TypstDocument`

```ts
TypstDocument.setWasmModuleInitOptions({
  getModule: () =>
    'http://localhost:20810/typst_ts_renderer_bg.wasm',
});
```

The default value is:

```ts
{
  beforeBuild: [],
  getModule: () => '/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
}
```

=== Example: show document

See #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/typst.react/src/demo")[typst.react demo] for more details.
