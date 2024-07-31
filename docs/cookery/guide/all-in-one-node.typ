#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-one Library for Node.js")

#include "claim.typ"

The compiler and renderer are integrated into a same node library for simpler and cleaner APIs, since there is no urgent need to tree-shake the components in node.js applications.

== Creating a Compiler

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

== Caution: Cleaning Global Cache

Please evict the global compilation cache periodically to avoid memory leak:

```ts
// A suggested `max_age` value for regular non-watch tools is `10`.
// A suggested `max_age` value for regular watch tools is `30`.
NodeCompiler.evictCache(10);
```

If you have some ideas about how to improve the cache eviction strategy, please let us know.

== Compiling out a document instance by compile arguments

With an intermediate document content:

```ts
const docs = await compiler.compile({
  mainFileContent: 'Hello, typst!',
});
```

With a file path:

```ts
const docs = await compiler.compile({
  mainFilePath: '/path/to/main-file.typ',
});
```

With extra `sys.inputs`:

```ts
const docs = await compiler.compile({
  mainFileContent: '#sys.inputs',
  inputs: {
    'theme': 'dark',
  },
});
```

== Compilation

Get output in various format:

```ts
// As a raw document object:
compiler.compile({ mainFileContent });
// As a precompiled vector-format document.
compiler.vector({ mainFileContent });
// As PDF.
compiler.pdf({ mainFileContent });
// As SVG that suitable for SVG viewers.
compiler.plainSvg({ mainFileContent });
// As SVG that only fits for web browsers but contains more features, like text selection.
compiler.svg({ mainFileContent });
```

== Querying

Query the document instance by some selector, such as a typst label:

```ts
compiler.query({ mainFileContent }, { selector: '<some-label>' });
```

== Adding/Removing in-memory shadow files

Add extra *binary input files*:

```ts
compiler.mapShadow('/assets/tiger.png', /* Node's Buffer Type */ pngData);
```

Add some extra *input file*:

```ts
await $typst.addSource('/template.typ', templateContent);
```

Relationship between `addSource` and `mapShadow`. The `addSource` function will `mapShadow` the content of the file by encoding it internally:

```ts
// add a json file (utf8)
compiler.mapShadow('/template.typ', (new TextEncoder()).encode(templateContent));
// same as above
```

Remove a shadow source or binary file:

```ts
compiler.unmapShadow('/assets/data.json');
```

Clean up all shadow files for underlying access model:

```ts
compiler.resetShadow();
```

Note: this function will also clean all files added by `addSource`.

== Settings: Configuring fonts.

Order of `fontArgs` is important. The precedence is:
- The former elements of `fontArgs` have higher precedence.
- The latter elements of `fontArgs` have lower precedence.
- System fonts.
- Default embedded fonts.

Load fonts from some directory:

```ts
const compiler = NodeCompiler.create({
  fontArgs: [
    { fontPaths: ['assets/fonts'] },
  ]
});
```

Load fonts by some blob:

```ts
const compiler = NodeCompiler.create({
  fontArgs: [
    // Node Buffer
    { fontBlobs: [someFontBlob] },
  ]
});
```
