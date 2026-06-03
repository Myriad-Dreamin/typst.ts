import { describe, it } from 'vitest';
import fileData from '../../../../fuzzers/corpora/visualize/gradient-radial-relative-self.artifact.sir.in?url&inline';
import { testSvg, makeSnapshot, testCanvas } from '../../tests/test-main.mjs';

const fileName = 'visualize/gradient-radial-relative-self';
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
