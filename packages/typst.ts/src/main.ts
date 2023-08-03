import * as initOptions from './options.init';
export type { InitOptions, BeforeBuildFn } from './options.init';
export type {
  RenderByContentOptions,
  RenderInSessionOptions,
  RenderPageOptions,
  RenderOptions,
} from './options.render';
export { preloadRemoteFonts, preloadSystemFonts } from './options.init';
import * as renderer from './renderer';
export type { TypstRenderer } from './renderer';
export { rendererBuildInfo, createTypstRenderer, createTypstSvgRenderer } from './renderer';
import { RenderView, renderTextLayer } from './view';
import * as compiler from './compiler';
import { FetchAccessModel } from './fs';
export { FetchAccessModel } from './fs';
export type { FetchAccessOptions } from './fs';
export type { TypstCompiler } from './compiler';
export { createTypstCompiler } from './compiler';

// Export module on window.
// todo: graceful way?
if (window) {
  (window as any).TypstRenderModule = {
    RenderView,
    renderTextLayer,

    createTypstRenderer: renderer.createTypstRenderer,
    createTypstSvgRenderer: renderer.createTypstSvgRenderer,
    preloadRemoteFonts: initOptions.preloadRemoteFonts,
    preloadSystemFonts: initOptions.preloadSystemFonts,
  };
  (window as any).TypstCompileModule = {
    createTypstCompiler: compiler.createTypstCompiler,
    preloadRemoteFonts: initOptions.preloadRemoteFonts,
    preloadSystemFonts: initOptions.preloadSystemFonts,

    FetchAccessModel,

    withAccessModel: initOptions.withAccessModel,
  };
}
