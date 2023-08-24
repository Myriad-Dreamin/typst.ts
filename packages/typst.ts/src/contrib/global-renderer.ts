import type { InitOptions } from '../options.init';
import type { TypstRenderer } from '../renderer';

let globalRenderer: TypstRenderer | undefined = undefined;
let globalRendererInitReady: Promise<TypstRenderer>;
let isReady = false;

export function getGlobalRenderer(): TypstRenderer | undefined {
  return isReady ? globalRenderer : undefined;
}

export function createGlobalRenderer(
  creator: (pdf: /* typeof pdfjsModule */ any) => TypstRenderer,
  pdf: /* typeof pdfjsModule */ any,
  initOptions: InitOptions,
): Promise<TypstRenderer> {
  // todo: determine renderer thread-safety
  // todo: check inconsistent initOptions
  const renderer = globalRenderer || creator(pdf);

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
  creator: (pdf: /* typeof pdfjsModule */ any) => TypstRenderer,
  pdf: /* typeof pdfjsModule */ any,
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
