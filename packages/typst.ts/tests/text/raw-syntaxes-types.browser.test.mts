import { describe, it } from 'vitest';
import fileData from '../../../../fuzzers/corpora/text/raw-syntaxes-types.artifact.sir.in?url&inline';
import { testSvg, makeSnapshot, testCanvas } from '../test-main.mjs';

const fileName = 'text/raw-syntaxes-types';
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
