import { describe, expect, it } from 'vitest';
import { TypstSnippet } from './contrib/snippet.mjs';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import compilerUrl from '../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';

import lsRegular from '../../../assets/data/LibertinusSerif-Regular-subset.otf?inline';
import lsBold from '../../../assets/data/LibertinusSerif-Bold-subset.otf?inline';
import lsItalic from '../../../assets/data/LibertinusSerif-Italic-subset.otf?inline';
import lsBoldItalic from '../../../assets/data/LibertinusSerif-BoldItalic-subset.otf?inline';
import { CompileFormatEnum, createTypstCompiler } from './compiler.mjs';
import { disableDefaultFontAssets, withPdfOptions } from './options.init.mjs';

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
    expect(data?.length).toMatchInlineSnapshot(`2141`);
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
    expect(data?.length).toMatchInlineSnapshot(`2502`);
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
    expect(data?.length).toMatchInlineSnapshot(`5080`);
  });
  it('should compile pdf', async () => {
    const data = await $typst.pdf({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`5674`);
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
    expect(data?.length).toMatchInlineSnapshot(`5864`);
  });
  it('should compile pdf 2', async () => {
    const data = await $typst.pdf({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`6147`);
  });
  it.skip('should compile svg 2', async () => {
    const data = await $typst.svg({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`19370`);
  });
});

describe('pdf export options', () => {
  // Bytes are searched via a byte-preserving decode; the conformance
  // declaration lives in the (uncompressed) XMP metadata stream.
  const containsAscii = (data: Uint8Array, text: string) =>
    new TextDecoder('latin1').decode(data).includes(text);
  const mainContent = '#set document(title: [Hello])\nHello, world!';

  it('should declare the requested standard', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets(), withPdfOptions({ pdfStandard: 'ua-1' })],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', mainContent);
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
      format: CompileFormatEnum.pdf,
    });
    expect(data.result).toBeDefined();
    expect(containsAscii(data.result!, 'pdfuaid')).toBe(true);
  });

  it('should declare no standard by default', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets()],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', mainContent);
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
      format: CompileFormatEnum.pdf,
    });
    expect(data.result).toBeDefined();
    expect(containsAscii(data.result!, 'pdfuaid')).toBe(false);
    // The baseline of accessibility: tagged even without a standard.
    expect(containsAscii(data.result!, 'StructTreeRoot')).toBe(true);
  });

  it('should disable tagging', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets(), withPdfOptions({ pdfTags: false })],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', mainContent);
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
      format: CompileFormatEnum.pdf,
    });
    expect(data.result).toBeDefined();
    expect(containsAscii(data.result!, 'StructTreeRoot')).toBe(false);
  });

  it('should use the creation timestamp', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets(), withPdfOptions({ creationTimestamp: 0 })],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', mainContent);
    const data = await compiler.compile({
      mainFilePath: '/main.typ',
      format: CompileFormatEnum.pdf,
    });
    expect(data.result).toBeDefined();
    // The document date is `auto` by default, so the timestamp lands in the
    // XMP metadata as the creation date.
    expect(containsAscii(data.result!, '1970-01-01T00:00:00')).toBe(true);
  });

  it('should support a per-world override', async () => {
    const compiler = createTypstCompiler();
    await compiler.init({
      beforeBuild: [disableDefaultFontAssets()],
      getModule: getModule().compiler,
    });
    await compiler.addSource('/main.typ', mainContent);
    const data = await compiler.runWithWorld({ mainFilePath: '/main.typ' }, async world => {
      world.setPdfOptions({ pdfStandard: 'ua-1' });
      await world.compile();
      return await world.pdf();
    });
    expect(data.result).toBeDefined();
    expect(containsAscii(data.result!, 'pdfuaid')).toBe(true);
  });

  it('should support the snippet provider', async () => {
    const $typst = createOne(false);
    $typst.use(TypstSnippet.withPdfOptions({ pdfStandard: 'ua-1' }));
    const data = await $typst.pdf({ mainContent });
    expect(data).toBeDefined();
    expect(containsAscii(data!, 'pdfuaid')).toBe(true);
  });
});
