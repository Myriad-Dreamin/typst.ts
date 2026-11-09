import { readFileSync } from 'node:fs';
import { gzipSync } from 'node:zlib';

const paths = process.argv.slice(2);
if (paths.length !== 2) {
  throw new Error('Usage: node measure-wasm-size.mjs <default.wasm> <html.wasm>');
}
const sizes = paths.map(path => {
  const wasm = readFileSync(path);
  return [wasm.length, gzipSync(wasm, { level: 9 }).length];
});
console.log('| Build | WASM bytes | gzip -9 bytes |');
console.log('| --- | ---: | ---: |');
console.log(`| web,misc | ${sizes[0].join(' | ')} |`);
console.log(`| web,misc,html | ${sizes[1].join(' | ')} |`);
console.log(
  `| Increase | ${sizes[1]
    .map((size, i) => {
      const delta = size - sizes[0][i];
      return `${delta} (${((100 * delta) / sizes[0][i]).toFixed(2)}%)`;
    })
    .join(' | ')} |`,
);
