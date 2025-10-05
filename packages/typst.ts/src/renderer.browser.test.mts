/// <reference types="@vitest/browser/context" />

import { describe, expect, it } from 'vitest';
import { TypstSnippet } from './contrib/snippet.mjs';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import { page, commands } from '@vitest/browser/context';

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
  it('should success with good vector', async () => {
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
  it('should success with good vector 2', async () => {
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


console.log(isNode);
describe('snippet renderer', () => {

  const getRenderer = async () => {
    const r = createTypstRenderer();
    await r.init({ getModule: getModule().renderer });
    return r;
  };

  const svg = async () => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const files = await getFiles();
    const data = await files['skyzh-cv/main.artifact.sir.in'];
    const rendered = await renderer.runWithSession(async renderSession => {
      renderer.manipulateData({
        renderSession,
        action: 'reset',
        data,
      });
      return await renderer.renderSvg({
        renderSession,
      });
    });
    container.innerHTML = rendered;
    const width = Number.parseFloat((container.firstElementChild as any).dataset.width);
    const height = Number.parseFloat((container.firstElementChild as any).dataset.height);
    console.log(container.firstElementChild);
    page.viewport(width, height);
    document.body.appendChild(container);
    return container;
  };

  const canvas = async () => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const files = await getFiles();
    const data = await files['skyzh-cv/main.artifact.sir.in'];
    await renderer.runWithSession(async renderSession => {
      renderer.manipulateData({
        renderSession,
        action: 'reset',
        data,
      });
      const width = await renderSession.docWidth;
      const height = await renderSession.docHeight;
      page.viewport(width, height);
      return await renderer.renderToCanvas({
        renderSession,
        container,
      });
    });

    return container;
  };
  const makeSnapshot = async (s: HTMLElement, name: string) => {
    const snapshotPath = await page.screenshot({ save: true, path: `../screenshots/renderer/${name}` });
    const { createSnapshot } = commands as any;
    const ret = await createSnapshot(snapshotPath, name);
    console.log(ret);
    // screenshotHash, refHash
    expect(ret.screenshotHash).toEqual(ret.refHash);
  };
  it('should renderer svg', async () => {
    const s = await svg();
    await makeSnapshot(s, 'skyzh-cv/main.svg.png');
  });
  it('should renderer canvas', async () => {
    const s = await canvas();
    await makeSnapshot(s, 'skyzh-cv/main.canvas.png');
  });
});

// it('should compile vector 2', async () => {
//   const data = await $typst.vector({
//     mainContent: '= A bit different!',
//   });
//   expect(data?.length).toMatchInlineSnapshot(`376`);
// });
// it('should compile pdf 2', async () => {
//   const data = await $typst.pdf({
//     mainContent: '= A bit different!',
//   });
//   expect(data?.length).toMatchInlineSnapshot(`2472`);
// });
// it('should compile svg 2', async () => {
//   const data = await $typst.svg({
//     mainContent: '= A bit different!',
//   });
//   expect(data?.length).toMatchInlineSnapshot(`13448`);
// });
// });
