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
 *   - preloadRemoteFonts
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
   * - preloadRemoteFonts
   * - preloadSystemFonts
   * - withAccessModel
   */
  beforeBuild: BeforeBuildFn[];

  /**
   * callbacks to fetch the wasm module
   *
   * There are many ways to provide a wasm module, see
   * {@link WebAssemblyModuleRef} for more details. If you don't provide a wasm
   * module, the default module will be used.
   */
  getModule(): WebAssemblyModuleRef | Promise<WebAssemblyModuleRef>;
}

/** @internal */
const _textFonts: string[] = [
  'LinLibertine_R.ttf',
  'LinLibertine_RB.ttf',
  'LinLibertine_RBI.ttf',
  'LinLibertine_RI.ttf',
  'NewCMMath-Book.otf',
  'NewCMMath-Regular.otf',
  'NewCM10-Regular.otf',
  'NewCM10-Bold.otf',
  'NewCM10-Italic.otf',
  'NewCM10-BoldItalic.otf',
  'DejaVuSansMono.ttf',
  'DejaVuSansMono-Bold.ttf',
  'DejaVuSansMono-Oblique.ttf',
  'DejaVuSansMono-BoldOblique.ttf',
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
const _emojiFonts: string[] = ['TwitterColorEmoji.ttf', 'NotoColorEmoji.ttf'];

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
   * @default 'https://raw.githubusercontent.com/Myriad-Dreamin/typst/assets-fonts/''
   */
  assetUrlPrefix?: string;

  /**
   * custom fetcher
   * Note: the default fetcher for node.js does not cache any fonts
   * @default fetch
   */
  fetcher?: typeof fetch;
}

export interface LoadRemoteFontsOptions extends LoadRemoteAssetsOptions {}

/**
 * disable default font assets
 */
export function disableDefaultFontAssets(): BeforeBuildFn {
  return preloadRemoteFonts([], { assets: false });
}

/**
 * preload font assets
 */
export function preloadFontAssets(options?: LoadRemoteAssetsOptions): BeforeBuildFn {
  return preloadRemoteFonts([], options);
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
 * ```
 */
export function preloadRemoteFonts(
  userFonts: (string | Uint8Array)[],
  options?: LoadRemoteFontsOptions,
): BeforeBuildFn {
  const fonts = [...userFonts];
  if (
    options &&
    options?.assets !== false &&
    options?.assets?.length &&
    options?.assets?.length > 0
  ) {
    let assetUrlPrefix =
      options.assetUrlPrefix ??
      'https://raw.githubusercontent.com/Myriad-Dreamin/typst/assets-fonts';
    if (assetUrlPrefix[assetUrlPrefix.length - 1] !== '/') {
      assetUrlPrefix += '/';
    }
    const prefix = (f: string[]) => f.map(font => assetUrlPrefix + font);
    for (const asset of options.assets) {
      switch (asset) {
        case 'text':
          fonts.push(...prefix(_textFonts));
          break;
        case 'cjk':
          fonts.push(...prefix(_cjkFonts));
          break;
        case 'emoji':
          fonts.push(...prefix(_emojiFonts));
          break;
      }
    }
  }

  const loader = async (_: BeforeBuildMark, { ref, builder }: InitContext) => {
    if (options?.fetcher) {
      ref.setFetcher(options.fetcher);
    }
    await ref.loadFonts(builder, fonts);
  };
  loader._preloadRemoteFontOptions = options;
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
    loadFonts(builder: Builder, fonts: (string | Uint8Array)[]): Promise<void>;
  };
  builder: Builder;
}

// todo: search browser
// searcher.search_browser().await?;
