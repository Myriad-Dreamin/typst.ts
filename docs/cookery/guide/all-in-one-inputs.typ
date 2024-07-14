=== Example: get output from input

get output with *single input file*:

```ts
const mainContent = 'Hello, typst!';
// into vector format
await $typst.vector({ mainContent });
// into svg format
await $typst.svg({ mainContent });
// into pdf format
await $typst.pdf({ mainContent });
// into canvas operations
await $typst.canvas(div, { mainContent });
```

With some extra *input file*:

```ts
await $typst.addSource('/template.typ', templateContent);
```

With extra *binary input files*:

```ts
const encoder = new TextEncoder();
// add a json file (utf8)
compiler.mapShadow('/assets/data.json', encoder.encode(jsonData));
// remove a json file
compiler.unmapShadow('/assets/data.json');

// add an image file
const pngData = await fetch(...).arrayBuffer();
compiler.mapShadow('/assets/tiger.png', new Uint8Array(pngData));
```

clean up shadow files for underlying access model:

```ts
compiler.resetShadow();
```

Note: this function will also clean all files added by `addSource`.
