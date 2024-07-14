#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-one Library for Node.js")

== Create a Compiler

Creates a new compiler with default arguments:
```ts
const compiler = NodeCompiler.create();
```

Creates a new compiler with custom arguments:
```ts
const compiler = NodeCompiler.create({
  workspace: '/path/to/workspace',
});
```