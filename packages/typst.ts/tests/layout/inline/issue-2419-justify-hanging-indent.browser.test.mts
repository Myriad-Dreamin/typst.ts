import { describe, it } from 'vitest';
import fileData from '../../../../fuzzers/corpora/layout/inline/issue-2419-justify-hanging-indent.artifact.sir.in?url&inline';
import { testSvg, makeSnapshot, testCanvas } from '../../tests/test-main.mjs';

const fileName = 'layout/inline/issue-2419-justify-hanging-indent';
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
