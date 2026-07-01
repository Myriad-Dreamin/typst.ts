#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "Compiler in Node.js")

Use #cross-link("/guide/all-in-one-node.typ")[the Node.js library] when you want to compile Typst from Node.js. It runs the compiler and exporters through the native addon, so it is usually faster and simpler than the browser Wasm package in Node applications.

```ts
import { NodeCompiler } from '@myriaddreamin/typst-ts-node-compiler';

const compiler = NodeCompiler.create();
const pdf = await compiler.pdf({
  mainFileContent: 'Hello, typst!',
});
```

See #cross-link("/guide/all-in-one-node.typ")[All-in-one Library for Node.js] for workspace, font, `sys.inputs`, shadow file, cache, and export options.
