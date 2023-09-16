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
export type { RenderSession, TypstRenderer } from './renderer';
export { rendererBuildInfo, createTypstRenderer, createTypstSvgRenderer } from './renderer';
import { RenderView, renderTextLayer } from './render/canvas/view';
import * as compiler from './compiler';
import { FetchAccessModel } from './fs';
export { FetchAccessModel } from './fs';
export type { FetchAccessOptions } from './fs';
export type { TypstCompiler } from './compiler';
export { createTypstCompiler } from './compiler';

// Export module on window.
if (window) {
  (window as any).TypstRenderModule = {
    RenderView,
    renderTextLayer,

    createTypstRenderer: renderer.createTypstRenderer,
    createTypstSvgRenderer: renderer.createTypstRenderer,
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
