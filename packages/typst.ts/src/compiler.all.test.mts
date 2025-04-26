import { describe, expect, it } from 'vitest';
import { TypstSnippet } from './contrib/snippet.mjs';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import compilerUrl from '../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';

// nodejs
const isNode =
  typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const $typst = new TypstSnippet();
if (!isNode) {
  $typst.setCompilerInitOptions({
    getModule: () => compilerUrl,
  });
  $typst.setRendererInitOptions({
    getModule: () => rendererUrl,
  });
} else {
  const fsImport = (file: string) => {
    const fs = require('fs');
    const path = require('path');
    return fs.readFileSync(path.join(import.meta.dirname, file));
  };
  $typst.setCompilerInitOptions({
    getModule: () => fsImport('../../compiler/pkg/typst_ts_web_compiler_bg.wasm'),
  });
  $typst.setRendererInitOptions({
    getModule: () => fsImport('../../renderer/pkg/typst_ts_renderer_bg.wasm'),
  });
}
$typst.use(TypstSnippet.disableDefaultFontAssets());

describe('snippet compiler', () => {
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
  it('should compile svg', async () => {
    const data = await $typst.svg({
      mainContent: 'Hello, world!',
    });
    expect(data?.length).toMatchInlineSnapshot(`13351`);
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
  it('should compile svg 2', async () => {
    const data = await $typst.svg({
      mainContent: '= A bit different!',
    });
    expect(data?.length).toMatchInlineSnapshot(`13353`);
  });
});
