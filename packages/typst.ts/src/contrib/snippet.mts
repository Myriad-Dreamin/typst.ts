import type { CompileOptions, TypstCompiler } from '../compiler.mjs';
import {
  withPackageRegistry,
  withAccessModel,
  type BeforeBuildFn,
  type InitOptions,
  preloadFontAssets,
} from '../options.init.mjs';
import type { TypstRenderer, RenderSession } from '../renderer.mjs';
import type { RenderToCanvasOptions, RenderSvgOptions } from '../options.render.mjs';
import { MemoryAccessModel, type WritableAccessModel } from '../fs/index.mjs';
import { FetchPackageRegistry } from '../fs/package.mjs';
import { PackageRegistry, PackageSpec } from '../internal.types.mjs';

/**
 * Some function that returns a promise of value or just that value.
 */
type PromiseJust<T> = (() => Promise<T>) | T;

/**
 * The sweet options for compiling and rendering the document.
 */
export type SweetCompileOptions =
  | {
      /**
       * The path of the main file.
       */
      mainFilePath: string;
    }
  | {
      /**
       * The source content of the main file.
       */
      mainContent: string;
    };

/**
 * The sweet options for compiling and rendering the document.
 */
export type SweetRenderOptions =
  | SweetCompileOptions
  | {
      /**
       * The artifact data in vector format.
       */
      vectorData: Uint8Array;
    };

type Role = 'compiler' | 'renderer';

/**
 * The sweet snippet provider for bullding the compiler or renderer component.
 * See {@link TypstSnippet#use} for more details.
 */
export interface TypstSnippetProvider {
  key: string;
  forRoles: Role[];
  provides: BeforeBuildFn[];
}

const isNode =
  // @ts-ignore
  typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

/**
 * Convenient util class for compiling documents, which is a wrapper of the
 * {@link TypstCompiler} and {@link TypstRenderer}.
 *
 * Note: the interface of this class is less stable than {@link TypstCompiler}
 * and {@link TypstRenderer}.
 *
 * @example
 * Use the *global shared* compiler instance:
 *
 * ```typescript
 * import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
 * ```
 *
 * Note: if you want to compile multiple documents, you should create a new
 * instance for each compilation work or maintain the shared state on the
 * utility instance `$typst` carefully, because the compilation process will
 * change the state of that.
 *
 * @example
 * Create an instance of utility:
 *
 * ```typescript
 * const $typst = new TypstSnippet({
 *   // optional renderer instance
 *   renderer: enableRendering ?? (() => {
 *     return createGlobalRenderer(createTypstRenderer,
 *       undefined, initOptions);
 *   }),
 *   compiler() => {
 *     return createGlobalCompiler(createTypstCompiler,
 *       initOptions);
 *   }
 * });
 * ```
 */
export class TypstSnippet {
  /** @internal */
  private mainFilePath: string;
  /** @internal */
  private cc?: PromiseJust<TypstCompiler>;
  /** @internal */
  private ex?: PromiseJust<TypstRenderer>;

  /**
   * Create a new instance of {@link TypstSnippet}.
   * @param cc the compiler instance, see {@link PromiseJust} and {@link TypstCompiler}.
   * @param ex the renderer instance, see {@link PromiseJust} and {@link TypstRenderer}.
   *
   * @example
   *
   * Passes a global shared compiler instance that get initialized lazily:
   * ```typescript
   * const $typst = new TypstSnippet(() => {
   *  return createGlobalCompiler(createTypstCompiler, initOptions);
   * });
   *
   */
  constructor(options?: {
    compiler?: PromiseJust<TypstCompiler>;
    renderer?: PromiseJust<TypstRenderer>;
  }) {
    this.cc = options?.compiler;
    this.ex = options?.renderer;
    this.mainFilePath = '/main.typ';
    this.providers = [];
  }

  /**
   * Set lazy initialized compiler instance for the utility instance.
   * @param cc the compiler instance, see {@link PromiseJust} and {@link TypstCompiler}.
   */
  setCompiler(cc: PromiseJust<TypstCompiler>) {
    this.cc = cc;
  }

  /**
   * Get an initialized compiler instance from the utility instance.
   */
  async getCompiler() {
    return (typeof this.cc === 'function' ? (this.cc = await this.cc()) : this.cc)!;
  }

  /**
   * Set lazy initialized renderer instance for the utility instance.
   * @param ex the renderer instance, see {@link PromiseJust} and {@link TypstRenderer}.
   */
  setRenderer(ex: PromiseJust<TypstRenderer>) {
    this.ex = ex;
  }

  /**
   * Get an initialized renderer instance from the utility instance.
   */
  async getRenderer() {
    return typeof this.ex === 'function' ? (this.ex = await this.ex()) : this.ex;
  }

  /**
   * provider for bullding the compiler or renderer component.
   */
  private providers?: PromiseJust<TypstSnippetProvider>[];
  use(...providers: PromiseJust<TypstSnippetProvider>[]) {
    if (!this.providers) {
      throw new Error('already prepare uses for instances');
    }
    this.providers.push(...providers);
  }

  private async prepareUse() {
    if (!this.providers) {
      return;
    }

    const providers = await Promise.all(
      this.providers.map(p => (typeof p === 'function' ? p() : p)),
    );
    this.providers = [];

    if (
      $typst == this &&
      !providers.some(p => p.key.includes('package-registry') || p.key.includes('access-model'))
    ) {
      // Note: the default fetch backend always adds a withAccessModel(mem)
      if (isNode) {
        const escapeImport = new Function('m', 'return import(m)');
        try {
          const m = new MemoryAccessModel();
          const { default: request } = await escapeImport('sync-request');

          $typst.use(
            TypstSnippet.withAccessModel(m),
            TypstSnippet.fetchPackageBy(m, (_: unknown, path: string) => {
              const response = request('GET', path);

              if (response.statusCode === 200) {
                return response.getBody(undefined);
              }
              return undefined;
            }),
          );
        } catch (e) {}
      } else {
        $typst.use(TypstSnippet.fetchPackageRegistry());
      }
    }

    const providers2 = await Promise.all(
      this.providers.map(p => (typeof p === 'function' ? p() : p)),
    );

    const ccOptions = (this.ccOptions ||= {});
    const ccBeforeBuild = (ccOptions.beforeBuild ||= []);

    const exOptions = (this.exOptions ||= {});
    const exBeforeBuild = (exOptions.beforeBuild ||= []);

    for (const provider of [...providers, ...providers2]) {
      if (provider.forRoles.includes('compiler')) {
        this.requireIsUninitialized('compiler', this.cc, TypstSnippet.$buildC);
        ccBeforeBuild.push(...provider.provides);
      }
      if (provider.forRoles.includes('renderer')) {
        this.requireIsUninitialized('renderer', this.ex, TypstSnippet.$buildR);
        exBeforeBuild.push(...provider.provides);
      }
    }
    this.providers = undefined;
  }

  /**
   * Set accessl model for the compiler instance
   * @example
   *
   * use memory access model
   *
   * ```typescript
   * const m = new MemoryAccessModel();
   * $typst.use(TypstSnippet.withAccessModel(m));
   * ```
   */
  static withAccessModel(accessModel: WritableAccessModel): TypstSnippetProvider {
    return {
      key: 'access-model',
      forRoles: ['compiler'],
      provides: [withAccessModel(accessModel)],
    };
  }

  /**
   * Set package registry for the compiler instance
   * @example
   *
   * use a customized package registry
   *
   * ```typescript
   * const n = new NodeFetchPackageRegistry();
   * $typst.use(TypstSnippet.withPackageRegistry(n));
   * ```
   */
  static withPackageRegistry(registry: PackageRegistry): TypstSnippetProvider {
    return {
      key: 'package-registry',
      forRoles: ['compiler'],
      provides: [withPackageRegistry(registry)],
    };
  }

  /**
   * Set access model for the compiler instance
   *
   * @example
   *
   * use default (memory) access model
   *
   * ```typescript
   * $typst.use(await TypstSnippet.fetchPackageRegistry());
   * ```
   *
   * @example
   *
   * use external access model
   *
   * ```typescript
   * const m = new MemoryAccessModel();
   * $typst.use(TypstSnippet.withAccessModel(m), await TypstSnippet.fetchPackageRegistry(m));
   * ```
   */
  static fetchPackageRegistry(accessModel?: WritableAccessModel): TypstSnippetProvider {
    const m = accessModel || new MemoryAccessModel();
    const provides = [
      ...(accessModel ? [] : [withAccessModel(m)]),
      withPackageRegistry(new FetchPackageRegistry(m)),
    ];
    return {
      key: 'package-registry$fetch',
      forRoles: ['compiler'],
      provides,
    };
  }

  /**
   * Set access model for the compiler instance
   * @example
   *
   * use a customized fetcher
   *
   * ```typescript
   * import request from 'sync-request-curl';
   * const m = new MemoryAccessModel();
   * $typst.use(TypstSnippet.withAccessModel(m), await TypstSnippet.fetchPackageBy(m, (_, httpUrl) => {
   *   const response = request('GET', this.resolvePath(path), {
   *     insecure: true,
   *   });
   *
   *   if (response.statusCode === 200) {
   *     return response.getBody(undefined);
   *   }
   *   return undefined;
   * }));
   * ```
   */
  static fetchPackageBy(
    accessModel: WritableAccessModel,
    fetcher: (path: PackageSpec, defaultHttpUrl: string) => Uint8Array | undefined,
  ): TypstSnippetProvider {
    class HttpPackageRegistry extends FetchPackageRegistry {
      pullPackageData(path: PackageSpec): Uint8Array | undefined {
        return fetcher(path, this.resolvePath(path));
      }
    }
    return {
      key: 'package-registry$lambda',
      forRoles: ['compiler'],
      provides: [withPackageRegistry(new HttpPackageRegistry(accessModel))],
    };
  }

  /** @internal */
  ccOptions: Partial<InitOptions>;
  /**
   * Set compiler init options for initializing global instance {@link $typst}.
   * See {@link InitOptions}.
   */
  setCompilerInitOptions(options: Partial<InitOptions>) {
    this.requireIsUninitialized('compiler', this.cc, TypstSnippet.$buildC);
    this.ccOptions = options;
  }

  /** @internal */
  exOptions: Partial<InitOptions>;
  /**
   * Set renderer init options for initializing global instance {@link $typst}.
   * See {@link InitOptions}.
   */
  setRendererInitOptions(options: Partial<InitOptions>) {
    this.requireIsUninitialized('renderer', this.ex, TypstSnippet.$buildR);
    this.exOptions = options;
  }

  /** @internal */
  pdfjsModule: unknown | undefined = undefined;
  /**
   * Set pdf.js module for initializing global instance {@link $typst}.
   */
  setPdfjsModule(module: unknown) {
    this.requireIsUninitialized('renderer', this.ex, TypstSnippet.$buildR);
    this.pdfjsModule = module;
  }

  /**
   * Set shared main file path.
   */
  setMainFilePath(path: string) {
    this.mainFilePath = path;
  }

  /**
   * Get shared main file path.
   */
  getMainFilePath() {
    return this.mainFilePath;
  }

  /**
   * See {@link TypstCompiler#addSource}.
   */
  async addSource(path: string, content: string) {
    (await this.getCompiler()).addSource(path, content);
  }

  /**
   * See {@link TypstCompiler#resetShadow}.
   */
  async resetShadow() {
    (await this.getCompiler()).resetShadow();
  }

  /**
   * See {@link TypstCompiler#mapShadow}.
   */
  async mapShadow(path: string, content: Uint8Array) {
    (await this.getCompiler()).mapShadow(path, content);
  }

  /**
   * See {@link TypstCompiler#unmapShadow}.
   */
  async unmapShadow(path: string) {
    (await this.getCompiler()).unmapShadow(path);
  }

  /**
   * Compile the document to vector (IR) format.
   * See {@link SweetCompileOptions}.
   */
  async vector(o?: SweetCompileOptions) {
    const opts = await this.getCompileOptions(o);
    return (await this.getCompiler()).compile(opts);
  }

  /**
   * Compile the document to PDF format.
   * See {@link SweetCompileOptions}.
   */
  async pdf(o?: SweetCompileOptions) {
    const opts = await this.getCompileOptions(o);
    opts.format = 'pdf';
    return (await this.getCompiler()).compile(opts);
  }

  /**
   * Compile the document to SVG format.
   * See {@link SweetRenderOptions} and {@link RenderSvgOptions}.
   */
  async svg(o?: SweetRenderOptions & RenderSvgOptions) {
    return this.transientRender(o, (renderer, renderSession) =>
      renderer.renderSvg({
        ...o,
        renderSession,
      }),
    );
  }

  /**
   * Compile the document to canvas operations.
   * See {@link SweetRenderOptions} and {@link RenderToCanvasOptions}.
   */
  async canvas(
    container: HTMLElement,
    o?: SweetRenderOptions & Omit<RenderToCanvasOptions, 'container'>,
  ) {
    return this.transientRender(o, (renderer, renderSession) =>
      renderer.renderToCanvas({
        container,
        ...o,
        renderSession,
      }),
    );
  }

  private async getCompileOptions(opts?: SweetCompileOptions): Promise<CompileOptions> {
    if (opts === undefined) {
      return { mainFilePath: this.mainFilePath };
    } else if (typeof opts === 'string') {
      throw new Error(`please specify opts as {mainContent: '...'} or {mainFilePath: '...'}`);
    } else if ('mainFilePath' in opts) {
      return { ...opts };
    } else {
      this.addSource(this.mainFilePath, opts.mainContent);
      return { mainFilePath: this.mainFilePath };
    }
  }

  private async getVector(opts?: SweetRenderOptions): Promise<Uint8Array> {
    if (opts && 'vectorData' in opts) {
      return opts.vectorData;
    }

    const options = await this.getCompileOptions(opts);
    return (await this.getCompiler()).compile(options);
  }

  private async transientRender<T>(
    opts: SweetRenderOptions | undefined,
    f: (rr: TypstRenderer, session: RenderSession) => T,
  ): Promise<T> {
    const rr = await this.getRenderer();
    if (!rr) {
      throw new Error('does not provide renderer instance');
    }
    const data = await this.getVector(opts);
    return await rr.runWithSession(async session => {
      rr.manipulateData({
        renderSession: session,
        action: 'reset',
        data,
      });
      return f(rr, session);
    });
  }

  private requireIsUninitialized<T>(role: string, c: PromiseJust<T>, e?: PromiseJust<T>) {
    if (typeof c !== 'function') {
      throw new Error(`${role} has been initialized: ${c}`);
    }
    if (e && c != e) {
      throw new Error(`${role} instance is set to non default value`);
    }
  }

  /** @internal */
  static async $buildC(this: TypstSnippet) {
    // lazy import compile module
    const { createGlobalCompiler } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/contrib/global-compiler.mjs'
    )) as any as typeof import('./global-compiler.mjs');
    const { createTypstCompiler } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/compiler.mjs'
    )) as any as typeof import('../compiler.mjs');

    await this.prepareUse();
    return createGlobalCompiler(createTypstCompiler, this.ccOptions);
  }

  /** @internal */
  static async $buildR(this: TypstSnippet) {
    // lazy import renderer module
    const { createGlobalRenderer } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs'
    )) as any as typeof import('./global-renderer.mjs');
    const { createTypstRenderer } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/renderer.mjs'
    )) as any as typeof import('../renderer.mjs');

    const pdfjs = this.pdfjsModule || (typeof window !== 'undefined' && (window as any)?.pdfjsLib);
    await this.prepareUse();
    return createGlobalRenderer(createTypstRenderer, pdfjs, this.exOptions);
  }
}

/**
 * The lazy initialized global shared instance of {@link TypstSnippet}. See
 * {@link TypstSnippet} for more details.
 */
export const $typst = new TypstSnippet({
  compiler: TypstSnippet.$buildC,
  renderer: TypstSnippet.$buildR,
});
