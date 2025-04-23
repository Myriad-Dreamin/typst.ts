import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';

import { cachedFontInitOptions } from './cached-font-middleware.mjs';

async function main() {
  $typst.setCompilerInitOptions(await cachedFontInitOptions());

  // Note: You can also use NodeFetchPackageRegistry
  // import { MemoryAccessModel } from '@myriaddreamin/typst.ts/dist/cjs/fs/memory.cjs';
  // import { NodeFetchPackageRegistry } from '@myriaddreamin/typst.ts/dist/cjs/fs/package.node.cjs';
  // import request from 'sync-request-curl';
  // const m = new MemoryAccessModel();
  // $typst.use(
  //   TypstSnippet.withAccessModel(m),
  //   TypstSnippet.withPackageRegistry(new NodeFetchPackageRegistry(m, request)),
  // );

  // Expected: Error: already set some assess model before: MemoryAccessModel([object Object])
  // const m = new MemoryAccessModel();
  // $typst.use(TypstSnippet.withAccessModel(m), TypstSnippet.fetchPackageRegistry(m));

  const svg = await $typst.svg({
    mainContent: `
#import "@preview/example:0.1.0": add

Hello, typst!
Example package: add(1, 2) = #add(1, 2)`,
  });

  console.log('Renderer works exactly! The rendered SVG file:', svg.length);
}

main();
