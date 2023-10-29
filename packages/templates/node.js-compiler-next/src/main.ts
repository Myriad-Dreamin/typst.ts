// test import
import * as _1 from '@myriaddreamin/typst-ts-renderer';
import * as _2 from '@myriaddreamin/typst-ts-web-compiler';

import { createTypstCompiler, createTypstRenderer } from '@myriaddreamin/typst.ts';

import { preloadFontAssets } from '@myriaddreamin/typst.ts/dist/cjs/options.init.cjs';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'fs';
import * as path from 'path';
import { HttpsProxyAgent } from 'https-proxy-agent';
import { cachedFontInitOptoins } from './cached-font-middleware';

async function main() {
  const compiler = createTypstCompiler();
  await compiler.init(await cachedFontInitOptoins());

  compiler.addSource('/main.typ', 'Hello, typst!');
  const artifactData = await compiler.compile({
    mainFilePath: '/main.typ',
  });

  const renderer = createTypstRenderer();
  await renderer.init();
  const svg = await renderer.runWithSession(async session => {
    renderer.manipulateData({
      renderSession: session,
      action: 'reset',
      data: artifactData,
    });
    return renderer.renderSvgDiff({
      renderSession: session,
    });
  });

  console.log('Renderer works exactly! The rendered SVG file:', svg.length);
}

main();
