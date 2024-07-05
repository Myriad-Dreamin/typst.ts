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

get output with *multiple input files*:
0

```ts
// the default value of main path is '/main.typ'
await $typst.addSource('/main.typ', mainContent);

// set path to main file
const mainFilePath = '/tasks/1/main.typ';
await $typst.setMainFilePath(mainFilePath)
await $typst.addSource(mainFilePath, mainContent);
```

What is quite important is that, when you are running multiple tasks asynchronously or in parallel, the call pattern `await $typst.xxx({ mainContent });` is unsafe (introduces undefined behavior). Insteadly you should call compilation by specifying path to the main file:

```ts
const mainFilePath = '/tasks/1/main.typ';
await $typst.addSource(mainFilePath, mainContent);

// compile source of path
await $typst.svg({ mainFilePath });
```

get output with *binary input files*:

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
