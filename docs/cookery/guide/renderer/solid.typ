#import "/docs/cookery/book.typ": book-page
#import "/docs/cookery/term.typ" as term

#show: book-page.with(title: "Solid Library")

= Solid Library


```typescript
import { TypstDocument } from "@myriaddreamin/typst.solid";
import { createResource } from "solid-js";

export const App = (artifact: Uint8Array) => {
  const getArtifactData = async () => {
    const response = await fetch(
      "http://localhost:3000/readme.artifact.sir.in"
    ).then((response) => response.arrayBuffer());

    return new Uint8Array(response);
  };
  const [vec] = createResource(getArtifactData);

  return (
    <div>
      <h1>Demo: Embed Your Typst Document in Solid</h1>
      <TypstDocument fill="#343541" artifact={vec()} />
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

#include "get-artifact.typ"

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
  getModule: () => 'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
}
```

=== Example: show document

See #link("https://github.com/Myriad-Dreamin/typst.ts/tree/main/packages/typst.solid/demo")[typst.solid demo] for more details.
