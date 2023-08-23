// @ts-ignore
import type * as typstRenderer from '@myriaddreamin/typst-ts-renderer';
import type * as typstCompiler from '@myriaddreamin/typst-ts-web-compiler';
import type { FsAccessModel } from './internal.types';
import type { WebAssemblyModuleRef } from './wasm';

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
export type BeforeBuildFn = StagedOptFn<BeforeBuildMark, unknown>;

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
      const fonts: {
        family: string;
        blob(): Promise<Blob>;
      }[] = await (window as any).queryLocalFonts();

      byFamily = byFamily ?? [];

      for (const font of fonts) {
        if (!byFamily.includes(font.family)) {
          continue;
        }

        const data: ArrayBuffer = await (await font.blob()).arrayBuffer();
        await builder.add_raw_font(new Uint8Array(data));
      }
    }

    const t2 = performance.now();
    console.log('preload system font time used:', t2 - t);
  };
}

export function withAccessModel(accessModel: FsAccessModel): BeforeBuildFn {
  return async (_, { builder }: InitContext) => {
    return new Promise(resolve => {
      builder.set_access_model(
        accessModel,
        (path: string) => {
          const lastModified = accessModel.getMTime(path);
          if (lastModified) {
            return lastModified.getTime();
          }
          return 0;
        },
        (path: string) => {
          return accessModel.isFile(path) || false;
        },
        (path: string) => {
          return accessModel.getRealPath(path) || path;
        },
        (path: string) => {
          return accessModel.readAll(path);
        },
      );
      resolve();
    });
  };
}

// todo: search browser
// searcher.search_browser().await?;

type Builder = typstRenderer.TypstRendererBuilder & typstCompiler.TypstCompilerBuilder;

interface InitContext {
  ref: {
    loadFont(builder: Builder, fontPath: string): Promise<void>;
  };
  builder: Builder;
}
