#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Angular Library")

= Angular Library

Use #link("https://www.npmjs.com/package/@myriaddreamin/typst.angular")[`@myriaddreamin/typst.angular`].

Import the angular module containing the `typst-document` component.

```typescript
/// component.module.ts
import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
```

And use directive `typst-document` in your template file.

```html
<typst-document props></typst-document>
```

== The `typst-document` component

=== Typical usage

```html
<typst-document
  fill="#343541"
  artifact="{{ artifact }}">
</typst-document>
```

=== `fill` property

Fill document with color.

```html
<typst-document fill="#343541">
</typst-document>
```

Note: Current typst.ts doesn't support a transparent background color in some browsers.

=== `artifact` property

Render the document with artifact from precompiler.

```html
<typst-document artifact="{{ artifact }}">
</typst-document>
```

The artifact can be only in #term.vector-format to this time.

To get `artifact` data, please refer to #term.ts-cli.

#include "pdfjs.typ"

=== Set renderer initialization option for `typst-document`

Retrieve a #term.init-option for initializating the renderer for `typst-document`

```ts
typst-document.setWasmModuleInitOptions({
  getModule: () =>
    'http://localhost:20810/typst_ts_renderer_bg.wasm',
});
```

The default value is:

```ts
{
  beforeBuild: [],
  getModule: () => '/assets/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
}
```

=== Example: show document

See #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/typst.angular/projects/demo")[typst.angular demo] for more details.
