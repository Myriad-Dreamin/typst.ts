import * as initOptions from './options.init';
export type { TypstRendererInitOptions, BeforeBuildFn } from './options.init';
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
