import type { InitOptions } from '../options.init.mjs';
import type { TypstRenderer } from '../renderer.mjs';

let globalRenderer: TypstRenderer | undefined = undefined;
let globalRendererInitReady: Promise<TypstRenderer>;
let isReady = false;

export function getGlobalRenderer(): TypstRenderer | undefined {
  return isReady ? globalRenderer : undefined;
}

export function createGlobalRenderer(
  creator: () => TypstRenderer,
  initOptions?: Partial<InitOptions>,
): Promise<TypstRenderer> {
  // todo: determine renderer thread-safety
  // todo: check inconsistent initOptions
  const renderer = globalRenderer || creator();

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
  creator: () => TypstRenderer,
  initOptions: Partial<InitOptions> | undefined,
  resolve: (renderer: TypstRenderer) => void,
  reject?: (err: any) => void,
) {
  const renderer = getGlobalRenderer();
  if (renderer) {
    resolve(renderer);
    return;
  }

  createGlobalRenderer(creator, initOptions).then(resolve).catch(reject);
}
