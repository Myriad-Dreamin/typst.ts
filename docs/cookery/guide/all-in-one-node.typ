#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-one Library for Node.js")

#include "claim.typ"

*Note: the following content is for typst.ts >=v0.6.0, and some APIs may be unusble in v0.5.x*

The compiler and renderer are integrated into a same node library for simpler and cleaner APIs, since there is no urgent need to tree-shake the components in node.js applications. It also has better performance, because the compiler and renderer are native code.

```ts
await NodeCompiler.create().svg({
  mainContent: 'Hello, typst!' }))
```

The library simplifies the APIs comparing with the #cross-link("/guide/compiler/service.typ")[Rust library APIs]. For example, the above example calls the underlying components:

```rs
let verse = TypstSystemUniverse::new(default_ops);
verse.mapShadow(memory_id, Bytes::from_string("Hello, typst!"));
let world = verse.snapshot_with(Some(TaskInputs {
  entry: memory_id,
  ..TaskInputs::default()
}));
let doc = typst::compile(&world)?;
return typst_svg::svg_merged(&doc);
```

== Creating a Compiler

Creates a new compiler with default arguments:
```ts
const $typst = NodeCompiler.create();
```

== Configuring Root

Configures the root of workspace to path:

```ts
const $typst = NodeCompiler.create({
  workspace: '/path/to/workspace',
});
```

The `NodeCompiler` will resolve the *absolute path* of the path *if it is relative at the time of creation*.

We suggest always pass *absolute paths* as root to ensure the compiler to work as expected.

```ts
const $typst = NodeCompiler.create({
  workspace: resolve('some/relative/path'),
});
```

== Configuring Fonts

Load fonts from some directory:

```ts
const $typst = NodeCompiler.create({
  fontArgs: [
    { fontPaths: ['assets/fonts'] },
  ]
});
```

Load fonts by some blob:

```ts
const $typst = NodeCompiler.create({
  fontArgs: [
    // Node Buffer
    { fontBlobs: [someFontBlob] },
  ]
});
```

Order of `fontArgs` is important. The precedence is:
- The former elements of `fontArgs` have higher precedence.
- The latter elements of `fontArgs` have lower precedence.
- System fonts.
- Default embedded fonts.

For example, with following code, typst will uses fonts from `assets/fonts1` first, then the `someFontBlob` font, then `assets/fonts2`, then the system fonts, and finally the fonts embedded in the binary.

```ts
const $typst = NodeCompiler.create({
  fontArgs: [
    { fontPaths: ['assets/fonts1'] },
    { fontBlobs: [someFontBlob] },
    { fontPaths: ['assets/fonts2'] },
  ]
});
```

== Compiling Documents

With an intermediate document content:

```ts
const docs = await $typst.compile({
  mainFileContent: 'Hello, typst!',
});
```

With a file path:

```ts
const docs = await $typst.compile({
  mainFilePath: '/path/to/main-file.typ',
});
```

== Caution: Cleaning Global Cache

Please evict the *global* compilation cache periodically to avoid memory leak:

```ts
// A suggested `max_age` value for regular non-watch tools is `10`.
// A suggested `max_age` value for regular watch tools is `30`.
$typst.evictCache(10);
```

*global* means that if you have multiple `NodeCompiler` instances, eviction will only affect all of them. This is a limitation of current typst's implementation.

If you have ideas about how to improve the cache eviction strategy, please let us know.

== Passing `sys.inputs`

Configures `sys.inputs` with string pairs:

```ts
const $typst = NodeCompiler.create({
  inputs: {
    'theme': 'dark',
  },
});
```

You can also pass `sys.inputs` when compiling documents:

```ts
const docs = await $typst.compile({
  mainFileContent: '#sys.inputs',
  inputs: {
    'theme': 'light',
  },
});
```

Note that, it will not inherit the `inputs` from the `NodeCompiler` instance but replace with the new one. For example, the following code compiles with `sys.inputs = (Y: "v")` instead of `sys.inputs = (X: "u", Y: "v")`:

```ts
const $typst = NodeCompiler.create({ inputs: { 'X': 'u' } });
await $typst.svg({ inputs: { 'Y': 'v' }, mainFileContent: '#sys.inputs' });
```

== Exporting to Various Formats

Gets output in various format:

```ts
// As a precompiled vector-format document.
$typst.vector({ mainFileContent });
// As PDF.
$typst.pdf({ mainFileContent });
// As a SVG string that suitable for SVG viewers.
$typst.plainSvg({ mainFileContent });
// As a SVG string that only fits for web browsers but contains more features, like text selection.
$typst.svg({ mainFileContent });
// As a HTML string using the experimental HTML export.
$typst.html({ mainFileContent });
```

You can also compile once and export to multiple formats later:

```ts
// As a raw document object:
const doc = $typst.compile({ mainFileContent });

$typst.vector(doc);
$typst.pdf(doc);
$typst.svg(doc);
```

todo: document options.

== Using `try_html`

This is an experimental API resembling `html` but exposes an object for advanced uses.

```ts
const output = $typst.tryHtml({ mainFileContent });

// Prints diagnostics if any.
if (htmlResult.hasError()) {
  htmlResult.printDiagnostics();
  return;
}

/// Gets the title of the document.
const title = htmlResult.result!.title();
/// Gets the HTML (<html>) string.
const htmlContent = htmlResult.result!.html();
/// Gets the <body> string.
const body = htmlResult.result!.body();
/// Gets the <body> bytes (to avoid creating strings for Node.js).
const bodyBytes = htmlResult.result!.bodyBytes();
```

== Querying

Queries the document instance by some selector, such as a typst label:

```ts
$typst.query({ mainFileContent }, { selector: '<some-label>' });
```

== Adding/Removing in-memory shadow files

Adds extra *binary input files*:

```ts
$typst.mapShadow('/assets/tiger.png', /* Node's Buffer Type */ pngData);
```

Adds some extra *input file*:

```ts
await $typst.addSource('/template.typ', templateContent);
```

Relationship between `addSource` and `mapShadow`. The `addSource` function will `mapShadow` the content of the file by encoding it internally:

```ts
// add a json file (utf8)
$typst.mapShadow('/template.typ', (new TextEncoder()).encode(templateContent));
// same as above
```

Removes a shadow source or binary file:

```ts
$typst.unmapShadow('/assets/data.json');
```

Cleans up all shadow files for underlying access model:

```ts
$typst.resetShadow();
```

Note: this function will also clean all files added by `addSource`.

== Reusing filesystem reads across compilations

Note: Since v0.6.0-rc1

By default, the Node compiler resets filesystem read cache per compilation. some tools would like to reuse the filesystem reads across compilations because they batchly compile documents and ignores filesystem changes during the batch compilation. This is not well modeled, but there is an internal flag to avoid re-reading the filesystem.

```js
const doc = $typst.compile({ mainFileContent, resetRead: false });
```
