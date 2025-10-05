import { describe, expect, it } from 'vitest';
import { TypstSnippet } from './contrib/snippet.mjs';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import compilerUrl from '../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';

import lsRegular from '../../../assets/data/LibertinusSerif-Regular-subset.otf?inline';
import lsBold from '../../../assets/data/LibertinusSerif-Bold-subset.otf?inline';
import lsItalic from '../../../assets/data/LibertinusSerif-Italic-subset.otf?inline';
import lsBoldItalic from '../../../assets/data/LibertinusSerif-BoldItalic-subset.otf?inline';
import { createTypstCompiler } from './compiler.mjs';
import { disableDefaultFontAssets } from './options.init.mjs';

// This is to reduce test time.
createTypstCompiler._impl.defaultAssets = [];

// nodejs
const isNode =
  typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const fsImport = (file: string) => {
  const fs = require('fs');
  const path = require('path');
  return fs.readFileSync(path.join(import.meta.dirname, file));
};

const getModule = () => {
  if (isNode) {
    return {
      compiler: () => fsImport('../../compiler/pkg/typst_ts_web_compiler_bg.wasm'),
      renderer: () => fsImport('../../renderer/pkg/typst_ts_renderer_bg.wasm'),
    };
  }
  return {
    compiler: () => compilerUrl,
    renderer: () => rendererUrl,
  };
};

const createOne = (withFonts: boolean) => {
  const $typst = new TypstSnippet();
  $typst.setCompilerInitOptions({
    getModule: getModule().compiler,
  });
  $typst.setRendererInitOptions({
    getModule: getModule().renderer,
  });

  $typst.use(TypstSnippet.disableDefaultFontAssets());
  if (withFonts) {
    $typst.use(TypstSnippet.preloadFonts([lsRegular, lsBold, lsItalic, lsBoldItalic]));
  }
  return $typst;
};

describe('compiler creations', () => {
  it('should success with undefined options', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      getModule: getModule().compiler,
    });
  });
  it('should success with no options', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [],
      getModule: getModule().compiler,
    });
  });
  it('should success with good vector', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets()],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', 'Hello, world!');
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
    });
    expect(data.result?.length).toMatchInlineSnapshot(`368`);
  });
  it('should success with good vector', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets()],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', '= A bit different!');
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
    });
    expect(data.result?.length).toMatchInlineSnapshot(`376`);
  });
});

describe('snippet compiler', () => {
  const $typst = createOne(false);

  it('should compile vector', async () => {
    const data = await $typst.vector({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`368`);
  });
  it('should compile pdf', async () => {
    const data = await $typst.pdf({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`2222`);
  });
  it.skip('should compile svg', async () => {
    const data = await $typst.svg({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`13446`);
  });

  it('should compile vector 2', async () => {
    const data = await $typst.vector({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`376`);
  });
  it('should compile pdf 2', async () => {
    const data = await $typst.pdf({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`2472`);
  });
  it.skip('should compile svg 2', async () => {
    const data = await $typst.svg({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`13448`);
  });
});

describe('snippet compiler with fonts', () => {
  const $typst = createOne(true);

  it('should compile vector', async () => {
    const data = await $typst.vector({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`4976`);
  });
  it('should compile pdf', async () => {
    const data = await $typst.pdf({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`5527`);
  });
  it.skip('should compile svg', async () => {
    const data = await $typst.svg({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`18536`);
  });

  it('should compile vector 2', async () => {
    const data = await $typst.vector({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`5736`);
  });
  it('should compile pdf 2', async () => {
    const data = await $typst.pdf({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`5965`);
  });
  it.skip('should compile svg 2', async () => {
    const data = await $typst.svg({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`19370`);
  });
});
