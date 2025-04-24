#import "/docs/cookery/book.typ": *

#show: book-page.with(title: "All-in-one Library for Node.js")

#include "claim.typ"

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

Creates a new compiler with custom arguments:
```ts
const $typst = NodeCompiler.create({
  workspace: '/path/to/workspace',
});
```

== Configuring Root

Configures a workspace to some *absolute path*:

```ts
const $typst = NodeCompiler.create({
  workspace: '/path/to/workspace',
});
```

== Configuring `sys.inputs`

Configures `sys.inputs` with string pairs:

```ts
const $typst = NodeCompiler.create({
  inputs: {
    'theme': 'dark',
  },
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

With extra `sys.inputs`:

```ts
const docs = await $typst.compile({
  mainFileContent: '#sys.inputs',
  inputs: {
    'theme': 'dark',
  },
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

== Export to Various Formats

Get output in various format:

```ts
// As a precompiled vector-format document.
$typst.vector({ mainFileContent });
// As PDF.
$typst.pdf({ mainFileContent });
// As SVG that suitable for SVG viewers.
$typst.plainSvg({ mainFileContent });
// As SVG that only fits for web browsers but contains more features, like text selection.
$typst.svg({ mainFileContent });
```

You can also compile once and export to multiple formats later:

```ts
// As a raw document object:
const doc = $typst.compile({ mainFileContent });

$typst.vector(doc);
$typst.pdf(doc);
$typst.plainSvg(doc);
$typst.svg(doc);
```

todo: document options.

== Querying

Query the document instance by some selector, such as a typst label:

```ts
$typst.query({ mainFileContent }, { selector: '<some-label>' });
```

== Adding/Removing in-memory shadow files

Add extra *binary input files*:

```ts
$typst.mapShadow('/assets/tiger.png', /* Node's Buffer Type */ pngData);
```

Add some extra *input file*:

```ts
await $typst.addSource('/template.typ', templateContent);
```

Relationship between `addSource` and `mapShadow`. The `addSource` function will `mapShadow` the content of the file by encoding it internally:

```ts
// add a json file (utf8)
$typst.mapShadow('/template.typ', (new TextEncoder()).encode(templateContent));
// same as above
```

Remove a shadow source or binary file:

```ts
$typst.unmapShadow('/assets/data.json');
```

Clean up all shadow files for underlying access model:

```ts
$typst.resetShadow();
```

Note: this function will also clean all files added by `addSource`.
