#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "Fonts")

#include "../claim.typ"

The web compiler does not embed fonts. It starts with a small set of default assets and, apart from
that, uses whatever fonts you register. This page covers hosting the defaults yourself, registering
your own fonts, and — when you offer many faces — fetching a font only once a document actually uses it.

== Default font assets

The default assets (_text, math and raw_ fonts, plus CJK and emoji, about 4.4 MB in total) are fetched
from our CDN on first use. For production you should download them once and serve them from your own
server:

```ts
import { $typst } from '@myriaddreamin/typst.ts';
import { TypstSnippet } from '@myriaddreamin/typst.ts/contrib/snippet';

$typst.use(TypstSnippet.preloadFontAssets({
  assets: ['text', 'cjk', 'emoji'],
  assetUrlPrefix: 'https://example.com/typst-assets/',
}));
```

The assets are the ones hosted at #link("https://github.com/Myriad-Dreamin/typst/tree/assets-fonts")[the assets-fonts branch];
`assets` also accepts a subset of `('text', 'cjk', 'emoji')`. If your documents never need them, turn
them off and register only your own fonts:

```ts
$typst.use(TypstSnippet.disableDefaultFontAssets());
```

== Registering your own fonts

The simplest way is to hand the bytes, or a URL, to the compiler before it is initialized:

```ts
$typst.use(TypstSnippet.preloadFontData(
  new Uint8Array(await (await fetch('/fonts/NotoSansSC-Regular.otf')).arrayBuffer()),
));
$typst.use(TypstSnippet.preloadFontFromUrl('/fonts/AnotherFont.ttf'));
```

In Node.js, `NodeCompiler.create` takes `fontArgs` instead (paths, blobs, and their precedence); see
#cross-link("/guide/all-in-one-node.typ")[All-in-One Library for Node.js].

== Loading a font only when a document uses it

Registering every face you offer is expensive: a single CJK face is a few megabytes, and a document
usually uses a handful of them. typst selects fonts from metadata it knows up front, so the trick is
to ship the _metadata_ cheaply and the _bytes_ lazily — register every face by its font info, and let
the compiler ask for the bytes of the faces it picks.

=== 1. Build a font index offline

`getFontInfo` gives you the metadata typst needs to select a face. Do this once per face, on your
build machine, and store `{ info, url }` next to the font files:

```ts
import { createTypstFontBuilder } from '@myriaddreamin/typst.ts/compiler';

const builder = createTypstFontBuilder();
await builder.init();

const info = await builder.getFontInfo(fontBytes); // store { info, url } for each face
```

A script that produces this index for a directory of fonts lives in gistd:
#link("https://github.com/Myriad-Dreamin/gistd/blob/e5662a756a70a43d6a7161d0f8fa14c701df14e5/scripts/font_info.ts#L88")[`scripts/font_info.ts`].

=== 2. Register each face lazily

`addLazyFont` takes the info from step 1 plus a function that returns the bytes when the compiler asks
for them. Registration itself downloads nothing:

```ts
import { loadFontSync } from '@myriaddreamin/typst.ts/init';

await builder.addLazyFont(info, loadFontSync({ info, url: '/fonts/NotoSansSC-Regular.otf' }));
```

`loadFontSync` reads the font through a synchronous `XMLHttpRequest`, because the compiler resolves
fonts synchronously; it is available in the browser only. In Node.js pass your own function returning
the bytes for the requested face.

=== 3. Hand the resolver to the compiler

```ts
await builder.build(resolver => compiler.setFonts(resolver));
```

Only the faces the document actually uses are fetched, and only when it first uses them.

The same lazy registration is available through the init options, which take a list of
`{ info, url }` entries:

```ts
import { loadFonts } from '@myriaddreamin/typst.ts';

init({
  beforeBuild: [
    loadFonts([{ info, url: '/fonts/NotoSansSC-Regular.otf' }]),
  ],
});
```
