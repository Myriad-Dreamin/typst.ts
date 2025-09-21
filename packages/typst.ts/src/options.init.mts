// @ts-ignore
import type * as typstRenderer from '@myriaddreamin/typst-ts-renderer';
import type * as typstCompiler from '@myriaddreamin/typst-ts-web-compiler';
import type { FsAccessModel, PackageRegistry, PackageSpec } from './internal.types.mjs';
import type { WebAssemblyModuleRef } from './wasm.mjs';

/**
 * staged options function
 * @template S - stage mark
 * @template T - context type
 */
export type StagedOptFn<S extends symbol, T = any> = (s: S, t: T) => Promise<void>;

/**
 * this mark is used to identify the beforeBuild stage
 * @description will not be used in runtime code
 */
const BeforeBuildSymbol: unique symbol = Symbol('beforeBuild');

/**
 * this mark is used to identify the beforeBuild stage
 * @description cannot be created by any runtime code
 */
export type BeforeBuildMark = typeof BeforeBuildSymbol;

/**
 * before build stage
 * @description possible created by:
 *   - loadFonts
 *   - preloadSystemFonts
 *   - withAccessModel
 *   - withPackageRegistry
 */
export type BeforeBuildFn = StagedOptFn<BeforeBuildMark>;

/**
 *
 * @property {BeforeBuildFn[]} beforeBuild - callbacks before build stage
 */
export interface InitOptions {
  /**
   * callbacks before build stage
   *
   * before build stage, the registered functions will be executed in order
   * possible options:
   * - loadFonts
   * - preloadSystemFonts
   * - withAccessModel
   */
  beforeBuild: BeforeBuildFn[];

  /**
   * callbacks to fetch the wasm module wrapper
   */
  getWrapper?(): Promise<any>;

  /**
   * callbacks to fetch the wasm module
   *
   * There are many ways to provide a wasm module, see
   * {@link WebAssemblyModuleRef} for more details. If you don't provide a wasm
   * module, the default module will be used.
   */
  getModule(): WebAssemblyModuleRef | Promise<WebAssemblyModuleRef>;
}

export type LazyFont = {
  info: any;
} & (
    | {
      blob: (index: number) => Uint8Array;
    }
    | {
      url: string;
    }
  );

/** @internal */
const _textFonts: string[] = [
  'DejaVuSansMono-Bold.ttf',
  'DejaVuSansMono-BoldOblique.ttf',
  'DejaVuSansMono-Oblique.ttf',
  'DejaVuSansMono.ttf',
  'LibertinusSerif-Bold.otf',
  'LibertinusSerif-BoldItalic.otf',
  'LibertinusSerif-Italic.otf',
  'LibertinusSerif-Regular.otf',
  'LibertinusSerif-Semibold.otf',
  'LibertinusSerif-SemiboldItalic.otf',
  'NewCM10-Bold.otf',
  'NewCM10-BoldItalic.otf',
  'NewCM10-Italic.otf',
  'NewCM10-Regular.otf',
  'NewCMMath-Bold.otf',
  'NewCMMath-Book.otf',
  'NewCMMath-Regular.otf',
];
/** @internal */
const _cjkFonts: string[] = [
  'InriaSerif-Bold.ttf',
  'InriaSerif-BoldItalic.ttf',
  'InriaSerif-Italic.ttf',
  'InriaSerif-Regular.ttf',
  'Roboto-Regular.ttf',
  'NotoSerifCJKsc-Regular.otf',
];
/** @internal */
const _emojiFonts: string[] = ['TwitterColorEmoji.ttf', 'NotoColorEmoji-Regular-COLR.subset.ttf'];

type AvailableFontAsset = 'text' | 'cjk' | 'emoji';

export interface LoadRemoteAssetsOptions {
  /**
   * preload font assets or don't preload any font assets
   * @default ['text']
   */
  assets?: AvailableFontAsset[] | false;

  /**
   * customize url prefix for default assets from remote
   *
   * The default assets are hosted on github, you can download them and host
   * them on your own server, which is more practical for production.
   *
   * Hosted at: https://github.com/Myriad-Dreamin/typst/tree/assets-fonts
   * List of assets:
   * See {@link _textFonts}, {@link _cjkFonts}, and {@link _emojiFonts}
   *
   * @default 'jsdelivr-url of typst-assets and typst-dev-assets'
   */
  assetUrlPrefix?: string | Record<string, string>;

  /**
   * custom fetcher
   * Note: the default fetcher for node.js does not cache any fonts
   * @default fetch
   */
  fetcher?: typeof fetch;
}

export interface LoadRemoteFontsOptions extends LoadRemoteAssetsOptions { }

/**
 * disable default font assets
 */
export function disableDefaultFontAssets(): BeforeBuildFn {
  return loadFonts([], { assets: false });
}

/**
 * preload font assets
 */
export function preloadFontAssets(options?: LoadRemoteAssetsOptions): BeforeBuildFn {
  return loadFonts([], options);
}

export function _resolveAssets(options?: LoadRemoteFontsOptions) {
  const fonts = [];
  if (
    options &&
    options?.assets !== false &&
    options?.assets?.length &&
    options?.assets?.length > 0
  ) {
    let defaultPrefix: Record<string, string> = {
      text: 'https://cdn.jsdelivr.net/gh/typst/typst-assets@v0.13.1/files/fonts/',
      _: 'https://cdn.jsdelivr.net/gh/typst/typst-dev-assets@v0.13.1/files/fonts/',
    };
    let assetUrlPrefix = options.assetUrlPrefix ?? defaultPrefix;
    if (typeof assetUrlPrefix === 'string') {
      assetUrlPrefix = { _: assetUrlPrefix };
    } else {
      assetUrlPrefix = { ...defaultPrefix, ...assetUrlPrefix };
    }
    for (const key of Object.keys(assetUrlPrefix)) {
      const u = assetUrlPrefix[key];
      if (u[u.length - 1] !== '/') {
        assetUrlPrefix[key] = u + '/';
      }
    }

    const prefix = (asset: string, f: string[]) =>
      f.map(font => (assetUrlPrefix[asset] || assetUrlPrefix['_']) + font);
    for (const asset of options.assets) {
      switch (asset) {
        case 'text':
          fonts.push(...prefix(asset, _textFonts));
          break;
        case 'cjk':
          fonts.push(...prefix(asset, _cjkFonts));
          break;
        case 'emoji':
          fonts.push(...prefix(asset, _emojiFonts));
          break;
      }
    }
  }

  return fonts;
}

/**
 * @deprecated use {@link loadFonts} instead
 */
export function preloadRemoteFonts(
  userFonts: (string | Uint8Array)[],
  options?: LoadRemoteFontsOptions,
): BeforeBuildFn {
  return loadFonts(userFonts, options);
}

/**
 * load fonts
 *
 * @param fonts - url path to font files
 * @returns {BeforeBuildFn}
 * @example
 * ```ts
 * // preLoad fonts from remote url (because finto info is not provided)
 * import { init, loadFonts } from 'typst';
 * init({
 *   beforeBuild: [
 *     loadFonts([
 *      'https://fonts.gstatic.com/s/roboto/v27/KFOmCnqEu92Fr1Mu4mxKKTU1Kg.woff2', // remote url
 *      'dist/fonts/Roboto-Regular.ttf', // relative to the root of the website
 *     ]),
 *   ],
 * });
 * ```
 * @example
 * ```ts
 * // lazily Load fonts from remote url. The font information is obtained by `getFontInfo`
 * import { init, loadFonts } from 'typst';
 * init({
 *   beforeBuild: [
 *     loadFonts([
 *      {
 *        info: [...]
 *        url: 'https://fonts.gstatic.com/s/roboto/v27/KFOmCnqEu92Fr1Mu4mxKKTU1Kg.woff2';
 *      }
 *     ]),
 *   ],
 * });
 * ```
 */
export function loadFonts(
  userFonts: (string | Uint8Array | LazyFont)[],
  options?: LoadRemoteFontsOptions,
): BeforeBuildFn {
  const assetFonts = _resolveAssets(options);
  const loader = async (_: BeforeBuildMark, { ref, builder }: InitContext) => {
    if (options?.fetcher) {
      ref.setFetcher(options.fetcher);
    }
    await ref.loadFonts(builder, [...userFonts, ...assetFonts]);
  };
  loader._preloadRemoteFontOptions = options;
  loader._kind = 'fontLoader';
  return loader;
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

/**
 * (compile only) set pacoage registry
 *
 * @param accessModel: when compiling, the pacoage registry is used to access the
 * data of files
 * @returns {BeforeBuildFn}
 */
export function withPackageRegistry(packageRegistry: PackageRegistry): BeforeBuildFn {
  return async (_, { builder }: InitContext) => {
    return new Promise(resolve => {
      builder.set_package_registry(packageRegistry, function (spec: PackageSpec) {
        return packageRegistry.resolve(spec, this);
      });
      resolve();
    });
  };
}

/**
 * (compile only) set access model
 *
 * @param accessModel: when compiling, the access model is used to access the
 * data of files
 * @returns {BeforeBuildFn}
 */
export function withAccessModel(accessModel: FsAccessModel): BeforeBuildFn {
  return async (_, ctx: InitContext) => {
    if (ctx.alreadySetAccessModel) {
      throw new Error(
        `already set some assess model before: ${ctx.alreadySetAccessModel.constructor?.name}(${ctx.alreadySetAccessModel})`,
      );
    }
    ctx.alreadySetAccessModel = accessModel;
    return new Promise(resolve => {
      ctx.builder.set_access_model(
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

/**
 * @internal builder
 */
type Builder = typstRenderer.TypstRendererBuilder & typstCompiler.TypstCompilerBuilder;

/**
 * @internal build context
 */
interface InitContext {
  ref: {
    setFetcher(fetcher: typeof fetch): void;
    loadFonts(builder: Builder, fonts: (string | Uint8Array | LazyFont)[]): Promise<void>;
  };
  builder: Builder;
  alreadySetAccessModel: any;
}

// todo: search browser
// searcher.search_browser().await?;
