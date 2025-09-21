import type { CompileOptions, TypstCompiler, TypstFontBuilder } from '../compiler.mjs';
import {
  withPackageRegistry,
  withAccessModel,
  type BeforeBuildFn,
  type InitOptions,
  preloadFontAssets,
  disableDefaultFontAssets,
  loadFonts,
  LoadRemoteAssetsOptions,
} from '../options.init.mjs';
import { loadFontSync } from '../init.mjs';
import type { TypstRenderer, RenderSession } from '../renderer.mjs';
import type { RenderToCanvasOptions, RenderSvgOptions } from '../options.render.mjs';
import { MemoryAccessModel, type WritableAccessModel } from '../fs/index.mjs';
import { FetchPackageRegistry } from '../fs/package.mjs';
import {
  PackageRegistry,
  PackageSpec,
  SemanticTokens,
  SemanticTokensLegend,
} from '../internal.types.mjs';
import { randstr } from '../utils.mjs';

/**
 * Some function that returns a promise of value or just that value.
 */
type PromiseJust<T> = (() => Promise<T>) | T;

interface CompileOptionsCommon {
  /**
   * The root of the main file.
   */
  root?: string;
  /**
   * Adds a string key-value pair visible through `sys.inputs`
   *
   * Note: pass `{}` to clear `sys.inputs`
   *
   * Note: When passing `undefined`, compiler will use last set `sys.inputs`.
   *
   * Note: This means you should always specify inputs when using compiler for concurrent tasks.
   */
  inputs?: Record<string, string>;
}

/**
 * The sweet options for compiling and rendering the document.
 */
export type SweetCompileOptions = (
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
  }
) &
  CompileOptionsCommon;

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

export type SweetLazyFont = {
  info: any;
} & (
    | {
      blob: (index: number) => Uint8Array;
    }
    | {
      url: string;
    }
  );

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
 * import { $typst } from '@myriaddreamin/typst.ts';
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
  private fr?: PromiseJust<TypstFontBuilder>;
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
    fontResolver?: PromiseJust<TypstFontBuilder>;
    renderer?: PromiseJust<TypstRenderer>;
  }) {
    this.cc = options?.compiler || TypstSnippet.buildLocalCompiler;
    this.fr = options?.fontResolver || TypstSnippet.buildLocalFontResolver;
    this.ex = options?.renderer || TypstSnippet.buildLocalRenderer;
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

  async getFontResolver() {
    return (typeof this.fr === 'function' ? (this.fr = await this.fr()) : this.fr)!;
  }

  /**
   * Get an initialized compiler instance from the utility instance.
   */
  async getCompiler() {
    return (typeof this.cc === 'function' ? (this.cc = await this.cc()) : this.cc)!;
  }

  private async getCompilerReset() {
    const compiler = await this.getCompiler();
    await compiler.reset();
    return compiler;
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
  async getRenderer(): Promise<TypstRenderer> {
    return typeof this.ex === 'function' ? (this.ex = await this.ex()) : this.ex!;
  }

  private providers?: PromiseJust<TypstSnippetProvider>[];
  /**
   * add providers for bullding the compiler or renderer component.
   */
  use(...providers: PromiseJust<TypstSnippetProvider>[]) {
    if (!this.providers) {
      throw new Error('already prepare uses for instances');
    }
    this.providers.push(...providers);
  }

  /**
   * todo: add docs
   */
  static preloadFontFromUrl(fontUrl: string): TypstSnippetProvider {
    return TypstSnippet.preloadFonts([fontUrl]);
  }

  /**
   * todo: add docs
   */
  static preloadFontData(fontData: Uint8Array): TypstSnippetProvider {
    return TypstSnippet.preloadFonts([fontData]);
  }

  /**
   * todo: add docs
   */
  static preloadFonts(userFonts: (string | Uint8Array)[]): TypstSnippetProvider {
    return {
      key: 'access-model',
      forRoles: ['compiler'],
      provides: [loadFonts(userFonts)],
    };
  }

  /**
   * don't load any default font assets.
   * todo: add docs
   */
  static disableDefaultFontAssets(): TypstSnippetProvider {
    return {
      key: 'access-model',
      forRoles: ['compiler'],
      provides: [disableDefaultFontAssets()],
    };
  }

  /**
   * todo: add docs
   */
  static preloadFontAssets(options?: LoadRemoteAssetsOptions): TypstSnippetProvider {
    return {
      key: 'access-model',
      forRoles: ['compiler'],
      provides: [preloadFontAssets(options)],
    };
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
   * Retrieve an access model to store the data of fetched files.
   * Provide a PackageRegistry instance for the compiler instance.
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
   * Retrieve a fetcher for fetching package data.
   * Provide a PackageRegistry instance for the compiler instance.
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
    this.requireIsUninitialized('compiler', this.cc);
    this.ccOptions = options;
  }

  /** @internal */
  exOptions: Partial<InitOptions>;
  /**
   * Set renderer init options for initializing global instance {@link $typst}.
   * See {@link InitOptions}.
   */
  setRendererInitOptions(options: Partial<InitOptions>) {
    this.requireIsUninitialized('renderer', this.ex);
    this.exOptions = options;
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

  removeTmp(opts: CompileOptions): Promise<void> {
    if (opts.mainFilePath.startsWith('/tmp/')) {
      return this.unmapShadow(opts.mainFilePath);
    }

    return Promise.resolve();
  }

  /**
   * Adds a font to the compiler.
   *
   * @example
   *
   * ```typescript
   * const fonts = await fetch('fontInfo.json').then(res => res.json());
   * $typst.addFonts(fonts.map(font => $typst.loadFont(font.url)));
   * ```
   *
   * @param fontInfos the font infos to add.
   */
  async setFonts(fontInfos: SweetLazyFont[]) {
    const fb = await this.getFontResolver();
    for (const font of fontInfos) {
      await fb.addLazyFont(font, 'blob' in font ? font.blob : loadFontSync(font), font);
    }
    const compiler = await this.getCompiler();
    await fb.build(async fonts => compiler.setFonts(fonts));
  }

  /**
   * Add a source file to the compiler.
   * See {@link TypstCompiler#addSource}.
   */
  async addSource(path: string, content: string) {
    (await this.getCompiler()).addSource(path, content);
  }

  /**
   * Reset the shadow files.
   * Note: this function is independent to the {@link reset} function.
   * See {@link TypstCompiler#resetShadow}.
   */
  async resetShadow() {
    (await this.getCompiler()).resetShadow();
  }

  /**
   * Add a shadow file to the compiler.
   * See {@link TypstCompiler#mapShadow}.
   */
  async mapShadow(path: string, content: Uint8Array) {
    (await this.getCompiler()).mapShadow(path, content);
  }

  /**
   * Remove a shadow file from the compiler.
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
    const compiler = await this.getCompilerReset();
    return compiler
      .compile(opts)
      .then(res => res.result)
      .finally(() => this.removeTmp(opts));
  }

  /**
   * Compile the document to PDF format.
   * See {@link SweetCompileOptions}.
   */
  async pdf(o?: SweetCompileOptions) {
    const opts = await this.getCompileOptions(o);
    opts.format = 'pdf';
    const compiler = await this.getCompilerReset();
    return compiler
      .compile(opts)
      .then(res => res.result)
      .finally(() => this.removeTmp(opts));
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

  /**
   * Get semantic tokens for the document.
   */
  async query<T>(o: SweetCompileOptions & { selector: string; field?: string }): Promise<T> {
    const opts = await this.getCompileOptions(o);
    const compiler = await this.getCompilerReset();
    return compiler
      .query<T>({
        ...o,
        ...opts,
      })
      .finally(() => this.removeTmp(opts));
  }

  /**
   * Get token legend for semantic tokens.
   */
  async getSemanticTokenLegend(): Promise<SemanticTokensLegend> {
    const compiler = await this.getCompilerReset();
    return compiler.getSemanticTokenLegend();
  }

  /**
   * Get semantic tokens for the document.
   * See {@link SweetCompileOptions}.
   * See {@link TypstCompiler#getSemanticTokens}.
   */
  async getSemanticTokens(o: SweetCompileOptions & { resultId?: string }): Promise<SemanticTokens> {
    const opts = await this.getCompileOptions(o);
    const compiler = await this.getCompilerReset();
    return compiler
      .getSemanticTokens({
        mainFilePath: opts.mainFilePath,
        resultId: o.resultId,
      })
      .finally(() => this.removeTmp(opts));
  }

  private async getCompileOptions(
    opts?: SweetCompileOptions,
  ): Promise<CompileOptions<any, 'none'>> {
    if (opts === undefined) {
      return { mainFilePath: this.mainFilePath, diagnostics: 'none' };
    } else if (typeof opts === 'string') {
      throw new Error(`please specify opts as {mainContent: '...'} or {mainFilePath: '...'}`);
    } else if ('mainFilePath' in opts) {
      return { ...opts, diagnostics: 'none' };
    } else {
      const destFile = `/tmp/${randstr()}.typ`;
      await this.addSource(destFile, opts.mainContent);
      return { mainFilePath: destFile, inputs: opts.inputs, diagnostics: 'none' };
    }
  }

  private async getVector(o?: SweetRenderOptions): Promise<Uint8Array> {
    if (o && 'vectorData' in o) {
      return o.vectorData;
    }

    const opts = await this.getCompileOptions(o);
    return (await this.getCompiler())
      .compile(opts)
      .then(res => res.result!)
      .finally(() => this.removeTmp(opts));
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

  prepareUseOnce: Promise<void> | undefined = undefined;
  private async prepareUse() {
    if (this.prepareUseOnce) {
      return this.prepareUseOnce;
    }
    return (this.prepareUseOnce = this.doPrepareUse());
  }

  private async doPrepareUse() {
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
        } catch (e) { }
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
        this.requireIsUninitialized('compiler', this.cc);
        ccBeforeBuild.push(...provider.provides);
      }
      if (provider.forRoles.includes('renderer')) {
        this.requireIsUninitialized('renderer', this.ex);
        exBeforeBuild.push(...provider.provides);
      }
    }
    this.providers = undefined;
  }

  private requireIsUninitialized<T>(role: string, c: PromiseJust<T>, e?: PromiseJust<T>) {
    if (c && typeof c !== 'function') {
      throw new Error(`${role} has been initialized: ${c}`);
    }
  }

  /** @internal */
  static async buildLocalCompiler(this: TypstSnippet) {
    const { createTypstCompiler } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/compiler'
    )) as any as typeof import('../compiler.mjs');

    await this.prepareUse();
    const compiler = createTypstCompiler();
    await compiler.init(this.ccOptions);
    return compiler;
  }

  /** @internal */
  static async buildLocalFontResolver(this: TypstSnippet) {
    const { createTypstFontBuilder } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/compiler'
    )) as any as typeof import('../compiler.mjs');

    await this.prepareUse();
    const fonts = createTypstFontBuilder();
    await fonts.init(this.ccOptions);
    return fonts;
  }

  /** @internal */
  static async buildGlobalCompiler(this: TypstSnippet) {
    // lazy import compile module
    const { createGlobalCompiler } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/contrib/global-compiler'
    )) as any as typeof import('./global-compiler.mjs');
    const { createTypstCompiler } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/compiler'
    )) as any as typeof import('../compiler.mjs');

    await this.prepareUse();
    return createGlobalCompiler(createTypstCompiler, this.ccOptions);
  }

  /** @internal */
  static async buildLocalRenderer(this: TypstSnippet) {
    const { createTypstRenderer } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/renderer'
    )) as any as typeof import('../renderer.mjs');

    await this.prepareUse();
    const renderer = createTypstRenderer();
    await renderer.init(this.exOptions);
    return renderer;
  }

  /** @internal */
  static async buildGlobalRenderer(this: TypstSnippet) {
    // lazy import renderer module
    const { createGlobalRenderer } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/contrib/global-renderer'
    )) as any as typeof import('./global-renderer.mjs');
    const { createTypstRenderer } = (await import(
      // @ts-ignore
      '@myriaddreamin/typst.ts/renderer'
    )) as any as typeof import('../renderer.mjs');

    await this.prepareUse();
    return createGlobalRenderer(createTypstRenderer, this.exOptions);
  }
}

/**
 * The lazy initialized global shared instance of {@link TypstSnippet}. See
 * {@link TypstSnippet} for more details.
 */
export const $typst = new TypstSnippet({
  compiler: TypstSnippet.buildGlobalCompiler,
  renderer: TypstSnippet.buildGlobalRenderer,
});
