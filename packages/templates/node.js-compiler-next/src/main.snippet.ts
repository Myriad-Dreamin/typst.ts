import { $typst } from '@myriaddreamin/typst.ts/dist/cjs/contrib/snippet.cjs';

import { cachedFontInitOptoins } from './cached-font-middleware';

async function main() {
  $typst.setCompilerInitOptions(await cachedFontInitOptoins());

  const svg = await $typst.svg({
    mainContent: 'Hello, typst!',
  });

  console.log('Renderer works exactly! The rendered SVG file:', svg.length);
}

main();
