import { describe, it } from 'vitest';
import fileData from '../../../../fuzzers/corpora/text/raw-highlight-html-jinja2.artifact.sir.in?url&inline';
import { testSvg, makeSnapshot, testCanvas } from '../test-main.mjs';

const fileName = 'text/raw-highlight-html-jinja2';
const getFile = async () => {
  return fetch(fileData).then(res => res.arrayBuffer()).then(buffer => new Uint8Array(buffer))
}

describe('renderer', () => {
  it(`should renderer svg`, async () => {
    await makeSnapshot(await testSvg(await getFile()), `${fileName}.svg.png`);
  });

  it('should renderer canvas', async () => {
    await makeSnapshot(await testCanvas(await getFile()), `${fileName}.canvas.png`);
  });
});
