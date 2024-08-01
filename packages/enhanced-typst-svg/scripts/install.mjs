import * as path from 'path';
import { writeFileSync, readFileSync } from 'fs';
import { projectRoot } from './common.mjs';

console.log(projectRoot);

const src = path.resolve(projectRoot, 'packages/enhanced-typst-svg/dist/index.min.js');
const srcData = readFileSync(src, 'utf8');

for (const inp of [
  'crates/conversion/vec2svg/src/frontend/typst.svg.js',
  'projects/hexo-renderer-typst/lib/svg_utils.cjs',
  'projects/typst-book/frontend/src/svg_utils.cjs',
]) {
  const dst = path.resolve(projectRoot, inp);

  console.log('index.min.js', '->', inp);

  let srcDataProc = srcData;
  if (inp.includes('typst.svg.js')) {
    // escape less-than sign etc for xml
    srcDataProc = srcDataProc.replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  // read data
  const dstData = readFileSync(dst, 'utf8');

  if (dstData === srcDataProc) {
    console.log('  same content, skipping');
    continue;
  }
  writeFileSync(dst, srcDataProc);
  console.log('  copied');
}
