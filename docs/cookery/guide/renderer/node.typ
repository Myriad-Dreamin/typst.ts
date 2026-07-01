#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "Renderer in Node.js")

Use #cross-link("/guide/all-in-one-node.typ")[the Node.js library] when you want to render Typst output from Node.js. The same `NodeCompiler` instance can compile source and export SVG, plain SVG, PDF, HTML, or the precompiled vector format.

```ts
import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';

const compiler = NodeCompiler.create();
const svg = await compiler.svg({
  mainFileContent: 'Hello, typst!',
});
```

If you already have a compiled document, you can reuse it for several export formats:

```ts
const doc = await compiler.compile({
  mainFileContent: 'Hello, typst!',
});

const svg = await compiler.svg(doc);
const pdf = await compiler.pdf(doc);
```
