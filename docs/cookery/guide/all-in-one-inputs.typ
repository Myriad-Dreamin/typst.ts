=== Compiling APIs

You can compile a *single typst code string* to different formats:

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

You can add some extra *source input files* before compiling:

```ts
await $typst.addSource('/template.typ', templateContent);
```

It also supports *binary input files*:

```ts
// add an image file
const pngData = await fetch(...).arrayBuffer();
$typst.mapShadow('/assets/tiger.png', new Uint8Array(pngData));
```

You can clean up shadow files for underlying access model:

```ts
$typst.resetShadow();
```

Note: this function will also clean all files added by `addSource`.
