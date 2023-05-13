import * as initOptions from './options.init';
export type { InitOptions, BeforeBuildFn } from './options.init';
export type {
  RenderByContentOptions,
  RenderInSessionOptions,
  RenderPageOptions,
  RenderOptions,
} from './options.render';
export { preloadRemoteFonts, preloadSystemFonts } from './options.init';
import * as driver from './driver';
export type { TypstRenderer } from './driver';
export { createTypstRenderer } from './driver';

// Export module on window.
// todo: graceful way?
if (window) {
  (window as any).TypstRenderModule = {
    createTypstRenderer: driver.createTypstRenderer,
    preloadRemoteFonts: initOptions.preloadRemoteFonts,
    preloadSystemFonts: initOptions.preloadSystemFonts,
  };
}
