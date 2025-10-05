
import { describe, expect, it } from 'vitest';
import { TypstSnippet } from './contrib/snippet.mjs';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';

import { createTypstRenderer } from './renderer.mjs';

const getFiles = () => {
  const files = import.meta.glob('../../../fuzzers/corpora/skyzh-cv/*.sir.in', {
    eager: true,
    query: '?url&inline',
    import: 'default'
  })

  const fileData = Object.entries(files).map(([key, value]) => {
    return {
      [key.replace(/^\.\.\/\.\.\/\.\.\/fuzzers\/corpora\//g, '')]: fetch(value as string).then(res => res.arrayBuffer()).then(buffer => new Uint8Array(buffer)),
    }
  })
  return Object.assign({}, ...fileData)
}

// nodejs
const isNode =
  typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const fsImport = (file: string) => {
  const fs = require('fs');
  const path = require('path');
  return fs.readFileSync(path.join(import.meta.dirname, file));
};

const getModule = () => {
  const compiler = () => {
    throw new Error("shouldn't load compiler when testing renderer")
  };
  if (isNode) {
    return {
      compiler,
      renderer: () => fsImport('../../renderer/pkg/typst_ts_renderer_bg.wasm'),
    };
  }
  return {
    compiler,
    renderer: () => rendererUrl,
  };
};

const createOne = () => {
  const $typst = new TypstSnippet();
  $typst.setCompilerInitOptions({
    getModule: getModule().renderer,
  });
  $typst.setRendererInitOptions({
    getModule: getModule().renderer,
  });

  $typst.use(TypstSnippet.disableDefaultFontAssets());
  return $typst;
};

describe('renderer creations', () => {
  it('should success with undefined options', async () => {
    const renderer = createTypstRenderer();
    await renderer.init({ getModule: getModule().renderer });
  });
  it('should success with no options', async () => {
    const renderer = createTypstRenderer();
    await renderer.init({
      beforeBuild: [],
      getModule: getModule().renderer,
    });
  });
  it.skip('should success with good vector', async () => {
    const renderer = createTypstRenderer();
    await renderer.init({ getModule: getModule().renderer });
    const files = await getFiles();
    const data = await files['skyzh-cv/main.artifact.sir.in'];
    const result = await renderer.runWithSession(async renderSession => {
      await renderer.manipulateData({
        renderSession,
        action: 'reset',
        data,
      });
      return renderer.renderSvg({
        renderSession,
      });
    });
    expect(result.length).toMatchInlineSnapshot(`194235`);
  });
  it.skip('should success with good vector 2', async () => {
    const renderer = createTypstRenderer();
    await renderer.init({ getModule: getModule().renderer });
    const files = await getFiles();
    const data = await files['skyzh-cv/main.white.artifact.sir.in'];
    const result = await renderer.runWithSession(async renderSession => {
      await renderer.manipulateData({
        renderSession,
        action: 'reset',
        data,
      });
      return renderer.renderSvg({
        renderSession,
      });
    });
    expect(result.length).toMatchInlineSnapshot(`365525`);
  });
  // todo: test invalid vector?
  // it('should success with good vector', async () => {
  //   const renderer = createTypstRenderer();
  //   await renderer.init({ getModule: getModule().renderer });
  //   await renderer.addSource('/main.typ', '= A bit different!');
  //   const data = await renderer.compile({
  //     mainFilePath: '/main.typ',
  //   });
  //   expect(data.result?.length).toMatchInlineSnapshot(`376`);
  // });
});

