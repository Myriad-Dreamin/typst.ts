import type { InitOptions } from '../options.init';
import type { TypstRenderer } from '../renderer';
import type * as pdfjsModule from 'pdfjs-dist';

let globalRenderer: TypstRenderer | undefined = undefined;
let globalRendererInitReady: Promise<TypstRenderer>;
let isReady = false;

export function getGlobalRenderer(): TypstRenderer | undefined {
  return isReady ? globalRenderer : undefined;
}

export function createGlobalRenderer(
  creator: (pdf: /* typeof pdfjsModule */ unknown) => TypstRenderer,
  pdf: /* typeof pdfjsModule */ unknown,
  initOptions: InitOptions,
): Promise<TypstRenderer> {
  // todo: determine renderer thread-safety
  // todo: check inconsistent initOptions
  const renderer = globalRenderer || creator(pdf as typeof pdfjsModule);

  if (globalRendererInitReady !== undefined) {
    return globalRendererInitReady;
  }

  return (globalRendererInitReady = (async () => {
    isReady = true;
    await renderer.init(initOptions);
    return (globalRenderer = renderer);
  })());
}

export function withGlobalRenderer(
  creator: (pdf: /* typeof pdfjsModule */ unknown) => TypstRenderer,
  pdf: /* typeof pdfjsModule */ unknown,
  initOptions: InitOptions,
  resolve: (renderer: TypstRenderer) => void,
  reject?: (err: any) => void,
) {
  const renderer = getGlobalRenderer();
  if (renderer) {
    resolve(renderer);
    return;
  }

  createGlobalRenderer(creator, pdf, initOptions).then(resolve).catch(reject);
}
