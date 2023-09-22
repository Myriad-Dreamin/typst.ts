// test import
import * as _1 from '@myriaddreamin/typst-ts-renderer';

import { createTypstCompiler, createTypstRenderer } from '@myriaddreamin/typst.ts';
import { existsSync, readFileSync } from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/// Note: this is only a example, please see
///   https://myriad-dreamin.github.io/typst.ts/cookery/guide/precompilers.html
/// to learn how to get the artifact of a typst document.
function retrieveArtifactData(): Uint8Array {
  return new Uint8Array(
    readFileSync(
      path.resolve(findGitRoot()!, 'fuzzers/corpora/skyzh-cv/main.artifact.sir.in'),
    ).buffer,
  );

  function findGitRoot() {
    let p = __dirname,
      lastP = '';
    while (p !== lastP) {
      if (existsSync(path.resolve(p, '.git/HEAD'))) {
        return p;
      }
      lastP = p;
      p = path.resolve(p, '..');
    }
    throw new Error('git root not found');
  }
}

async function main() {
  //   const compiler = createTypstCompiler();
  //   await compiler.init();

  //   compiler.compile()

  const renderer = createTypstRenderer();
  await renderer.init();
  const svg = await renderer.runWithSession(async session => {
    renderer.manipulateData({
      renderSession: session,
      action: 'reset',
      data: retrieveArtifactData(),
    });
    return renderer.renderSvgDiff({
      renderSession: session,
    });
  });

  console.log('Renderer works exactly! The rendered SVG file:', svg.length);
}

main();
