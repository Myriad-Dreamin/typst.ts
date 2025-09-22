import * as init from './init.mjs';
import * as initOptions from './options.init.mjs';
export type { InitOptions, BeforeBuildFn } from './options.init.mjs';
export type {
  RenderByContentOptions,
  RenderInSessionOptions,
  RenderCanvasOptions as RenderPageOptions,
  RenderOptions,
} from './options.render.mjs';
export { loadFonts, preloadRemoteFonts, preloadSystemFonts } from './options.init.mjs';
import * as renderer from './renderer.mjs';
export type { RenderSession, TypstRenderer } from './renderer.mjs';
export { rendererBuildInfo, createTypstRenderer } from './renderer.mjs';
import { RenderView } from './render/canvas/view.mjs';
import * as compiler from './compiler.mjs';
import { FetchAccessModel, MemoryAccessModel } from './fs/index.mjs';
import { FetchPackageRegistry } from './fs/package.mjs';
export { FetchAccessModel } from './fs/index.mjs';
export { FetchPackageRegistry } from './fs/package.mjs';
export type { FetchAccessOptions } from './fs/index.mjs';
export type { TypstCompiler } from './compiler.mjs';
export { createTypstCompiler } from './compiler.mjs';

// Export module on window.
if (typeof window !== 'undefined') {
  (window as any).TypstRenderModule = {
    RenderView,

    createTypstRenderer: renderer.createTypstRenderer,
    createTypstSvgRenderer: renderer.createTypstRenderer,
    preloadRemoteFonts: initOptions.loadFonts,
    loadFonts: initOptions.loadFonts,
    preloadSystemFonts: initOptions.preloadSystemFonts,
  };
  (window as any).TypstCompileModule = {
    createTypstCompiler: compiler.createTypstCompiler,
    createTypstFontBuilder: compiler.createTypstFontBuilder,
    preloadRemoteFonts: initOptions.loadFonts,
    loadFonts: initOptions.loadFonts,
    loadFontSync: init.loadFontSync,
    preloadSystemFonts: initOptions.preloadSystemFonts,

    FetchAccessModel,
    MemoryAccessModel,
    FetchPackageRegistry,

    withAccessModel: initOptions.withAccessModel,
    withPackageRegistry: initOptions.withPackageRegistry,
  };
}
