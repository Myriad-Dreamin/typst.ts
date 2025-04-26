import { expect, describe, it } from 'vitest';
import { _resolveAssets } from './options.init.mjs';

describe('resolve assets', () => {
  it('default', () => {
    const data = _resolveAssets();
    expect(data).toMatchInlineSnapshot(`[]`);
  });
  it('default2', () => {
    const data = _resolveAssets({});
    expect(data).toMatchInlineSnapshot(`[]`);
  });
  it('all', () => {
    const data = _resolveAssets({
      assets: ['text', 'cjk', 'emoji'],
    });
    expect(data).toMatchInlineSnapshot(`
      [
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-BoldOblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Oblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Semibold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-SemiboldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Book.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-BoldItalic.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Italic.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Regular.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/Roboto-Regular.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/NotoSerifCJKsc-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/TwitterColorEmoji.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/NotoColorEmoji.ttf",
      ]
    `);
  });
  it('text', () => {
    const data = _resolveAssets({
      assets: ['text'],
    });
    expect(data).toMatchInlineSnapshot(`
      [
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-BoldOblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Oblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Semibold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-SemiboldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Book.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Regular.otf",
      ]
    `);
  });
  it('text-cjk', () => {
    const data = _resolveAssets({
      assets: ['text', 'cjk'],
    });
    expect(data).toMatchInlineSnapshot(`
      [
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-BoldOblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Oblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Semibold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-SemiboldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Book.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-BoldItalic.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Italic.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/InriaSerif-Regular.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/Roboto-Regular.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/NotoSerifCJKsc-Regular.otf",
      ]
    `);
  });
  it('customized-url', () => {
    const data = _resolveAssets({
      assets: ['text', 'cjk'],
      assetUrlPrefix: 'https://my-server.com',
    });
    expect(data).toMatchInlineSnapshot(`
      [
        "https://my-server.com/DejaVuSansMono-Bold.ttf",
        "https://my-server.com/DejaVuSansMono-BoldOblique.ttf",
        "https://my-server.com/DejaVuSansMono-Oblique.ttf",
        "https://my-server.com/DejaVuSansMono.ttf",
        "https://my-server.com/LibertinusSerif-Bold.otf",
        "https://my-server.com/LibertinusSerif-BoldItalic.otf",
        "https://my-server.com/LibertinusSerif-Italic.otf",
        "https://my-server.com/LibertinusSerif-Regular.otf",
        "https://my-server.com/LibertinusSerif-Semibold.otf",
        "https://my-server.com/LibertinusSerif-SemiboldItalic.otf",
        "https://my-server.com/NewCM10-Bold.otf",
        "https://my-server.com/NewCM10-BoldItalic.otf",
        "https://my-server.com/NewCM10-Italic.otf",
        "https://my-server.com/NewCM10-Regular.otf",
        "https://my-server.com/NewCMMath-Bold.otf",
        "https://my-server.com/NewCMMath-Book.otf",
        "https://my-server.com/NewCMMath-Regular.otf",
        "https://my-server.com/InriaSerif-Bold.ttf",
        "https://my-server.com/InriaSerif-BoldItalic.ttf",
        "https://my-server.com/InriaSerif-Italic.ttf",
        "https://my-server.com/InriaSerif-Regular.ttf",
        "https://my-server.com/Roboto-Regular.ttf",
        "https://my-server.com/NotoSerifCJKsc-Regular.otf",
      ]
    `);
  });
  it('customized-url-record', () => {
    const data = _resolveAssets({
      assets: ['text', 'cjk'],
      assetUrlPrefix: {
        cjk: 'https://my-server.com',
      },
    });
    expect(data).toMatchInlineSnapshot(`
      [
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Bold.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-BoldOblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono-Oblique.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/DejaVuSansMono.ttf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-Semibold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/LibertinusSerif-SemiboldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-BoldItalic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Italic.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCM10-Regular.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Bold.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Book.otf",
        "https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/NewCMMath-Regular.otf",
        "https://my-server.com/InriaSerif-Bold.ttf",
        "https://my-server.com/InriaSerif-BoldItalic.ttf",
        "https://my-server.com/InriaSerif-Italic.ttf",
        "https://my-server.com/InriaSerif-Regular.ttf",
        "https://my-server.com/Roboto-Regular.ttf",
        "https://my-server.com/NotoSerifCJKsc-Regular.otf",
      ]
    `);
  });
});
