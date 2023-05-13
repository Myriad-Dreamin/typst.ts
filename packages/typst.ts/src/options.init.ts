// @ts-ignore
import type * as typst from '../../pkg/typst_ts_renderer';

/**
 * staged options function
 * @typedef StagedOptFn
 * @template S - stage mark
 * @template T - context type
 */
export type StagedOptFn<S extends symbol, T = any> = (s: S, t: T) => Promise<void>;

/**
 * this mark is used to identify the beforeBuild stage
 * @type {unique symbol}
 * @description will not be used in runtime code
 */
const BeforeBuildSymbol = Symbol('beforeBuild');

/**
 * this mark is used to identify the beforeBuild stage
 * @typedef BeforeBuildMark
 * @description cannot be created by any runtime code
 */
export type BeforeBuildMark = typeof BeforeBuildSymbol;

/**
 * before build stage
 * @typedef BeforeBuildFn
 * @description possible created by:
 *   - preloadRemoteFonts
 *   - preloadSystemFonts
 */
export type BeforeBuildFn = StagedOptFn<BeforeBuildMark, any>;

export type WebAssemblyModuleRef = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

/**
 *
 * @typedef InitOptions
 * @property {BeforeBuildFn[]} beforeBuild - callbacks before build stage
 */
export interface InitOptions {
  /**
   * callbacks before build stage
   *
   * before build stage, the registered functions will be executed in order
   * possible options:
   * - preloadRemoteFonts
   * - preloadSystemFonts
   */
  beforeBuild: BeforeBuildFn[];

  getModule(): WebAssemblyModuleRef | Promise<WebAssemblyModuleRef>;
}

/**
 * preload remote fonts
 *
 * @param fonts - url path to font files
 * @returns {BeforeBuildFn}
 * @example
 * ```typescript
 * import { init, preloadRemoteFonts } from 'typst';
 * init({
 *   beforeBuild: [
 *     preloadRemoteFonts([
 *      'https://fonts.gstatic.com/s/roboto/v27/KFOmCnqEu92Fr1Mu4mxKKTU1Kg.woff2', // remote url
 *      'dist/fonts/Roboto-Regular.ttf', // relative to the root of the website
 *     ]),
 *   ],
 * });
 */
export function preloadRemoteFonts(fonts: string[]): BeforeBuildFn {
  return async (_, { ref, builder }: InitContext) => {
    await Promise.all(fonts.map(font => ref.loadFont(builder, font)));
  };
}

/**
 * preload system fonts
 * @param byFamily - filter system fonts to preload by family name
 * @returns {BeforeBuildFn}
 * @example
 * ```typescript
 * import { init, preloadSystemFonts } from 'typst';
 * init({
 *   beforeBuild: [
 *     preloadSystemFonts({
 *       byFamily: ['Roboto'], // preload fonts by family name
 *     }),
 *   ],
 * });
 * ```
 */
export function preloadSystemFonts({ byFamily }: { byFamily?: string[] }): BeforeBuildFn {
  return async (_, { builder }: InitContext) => {
    const t = performance.now();

    if ('queryLocalFonts' in window) {
      const fonts = await (window as any).queryLocalFonts();
      console.log('local fonts count:', fonts.length);

      byFamily = byFamily ?? [];

      for (const font of fonts) {
        if (!byFamily.includes(font.family)) {
          continue;
        }

        console.log(font.family);

        const data: ArrayBuffer = await (await font.blob()).arrayBuffer();
        await builder.add_raw_font(new Uint8Array(data));
      }
    }

    const t2 = performance.now();
    console.log('font loading', t2 - t);
  };
}

// todo: search browser
// searcher.search_browser().await?;

interface InitContext {
  ref: {
    loadFont(builder: typst.TypstRendererBuilder, fontPath: string): Promise<void>;
  };
  builder: typst.TypstRendererBuilder;
}
