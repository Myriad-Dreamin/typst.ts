import { CanvasCommand } from './canvas-worker-common.mjs';

import type { RenderSession, TypstRendererDriver } from '../renderer.mjs';
type MainModule = typeof import('../main.mjs');

interface CanvasMessage extends Array<unknown> {
  [0]: CanvasCommand;
  [1]: any;
}

let renderer: TypstRendererDriver | null = null;

onmessage = async function (event: MessageEvent<CanvasMessage>) {
  const { data } = event;
  const [ty, opts] = data;
  switch (ty) {
    case CanvasCommand.Init: {
      (self as any).exports = {};
      // console.log('opts.mainScript', opts.mainScript);
      const m: MainModule = await import(opts.mainScript);
      // self.importScripts('http://localhost:20810/core/dist/cjs/main.bundle.js');
      // console.log('importScripts on init', Object.keys(m), m.createTypstRenderer);

      const rendererScript = import(
        'http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer.mjs' as any
      );
      const rendererWasm = fetch(
        'http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
      );
      renderer = m.createTypstRenderer() as TypstRendererDriver;
      await renderer.init({
        beforeBuild: [],
        getWrapper: () => rendererScript,
        getModule: () => rendererWasm,
      });
      // console.log('createTypstRenderer', renderer);
      // console.log('init', opts);

      postMessage([CanvasCommand.Init, null]);
      break;
    }
    case CanvasCommand.Render: {
      if (renderer === null) {
        console.error('renderer is null');
        return;
      }

      const { glyph } = opts;
      const canvas = new OffscreenCanvas(1024, 1024);
      // console.log('render', renderer, opts, '=>', canvas);

      const kernel = renderer.renderer;
      kernel.canvas_render_glyph(canvas, glyph);
      // canvas.commit();

      const img = canvas.transferToImageBitmap();
      postMessage([CanvasCommand.Render, { result: img }], { transfer: [img] });

      break;
    }
    default: {
      // console.log(event.data);
      break;
    }
  }
};
