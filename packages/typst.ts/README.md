# Typst.ts

Usage:

```typescript
import { $typst } from '@myriaddreamin/typst.ts';
console.log(
  (
    await $typst.svg({
      mainContent: 'Hello, typst!',
    })
  ).length,
);
// :-> 7317
```

See [Typst.ts](https://github.com/Myriad-Dreamin/typst.ts) and documentation for details:

- [Get Started](https://myriad-dreamin.github.io/typst.ts/cookery/get-started.html)
- [Compiler interfaces](https://myriad-dreamin.github.io/typst.ts/cookery/guide/compilers.html)

## Fonts in WOFF containers

The compiler consumes SFNT font data (`ttf` / `otf`). `woff` / `woff2` files are distribution containers: decode them with `woffToSfnt` from `@myriaddreamin/typst.ts/woff`, then register the resulting bytes like any other font.

```typescript
import { $typst, loadFonts } from '@myriaddreamin/typst.ts';
import { woffToSfnt } from '@myriaddreamin/typst.ts/woff';

const woffBytes = new Uint8Array(await (await fetch('/fonts/Regular.woff')).arrayBuffer());
$typst.setCompilerInitOptions({
  beforeBuild: [loadFonts([await woffToSfnt(woffBytes)])],
});

// WOFF2 needs a caller-supplied decoder, e.g. a wasm woff2 build:
// const sfnt = await woffToSfnt(woff2Bytes, { decodeWoff2: (bytes) => myWoff2Decoder.decode(bytes) });
```

