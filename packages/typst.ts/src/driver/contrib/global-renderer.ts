import { InitOptions } from '../options.init';
import { TypstRenderer, createTypstRenderer } from '../driver';
import type * as pdfjsModule from 'pdfjs-dist';

let globalRenderer: TypstRenderer | undefined = undefined;
let globalRendererInitReady: Promise<TypstRenderer>;
let isReady = false;

export function getGlobalRenderer(): TypstRenderer | undefined {
  return isReady ? globalRenderer : undefined;
}

export function createGlobalRenderer(
  pdf: typeof pdfjsModule,
  initOptions: InitOptions,
): Promise<TypstRenderer> {
  // todo: determine renderer thread-safety
  // todo: check inconsistent initOptions
  const renderer = globalRenderer || createTypstRenderer(pdf);

  if (globalRendererInitReady) {
    return globalRendererInitReady;
  }

  return (globalRendererInitReady = (async () => {
    isReady = true;
    await renderer.init(initOptions);
    return renderer;
  })());
}

export function withGlobalRenderer(
  pdf: typeof pdfjsModule,
  initOptions: InitOptions,
  resolve: (renderer: TypstRenderer) => void,
  reject?: (err: any) => void,
) {
  const renderer = getGlobalRenderer();
  if (renderer) {
    resolve(renderer);
    return;
  }

  createGlobalRenderer(pdf, initOptions).then(resolve).catch(reject);
}
