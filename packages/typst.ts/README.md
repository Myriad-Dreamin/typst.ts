# Typst.ts

Usage:

```typescript
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
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
