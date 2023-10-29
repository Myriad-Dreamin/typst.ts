import type { CompileOptions, TypstCompiler } from '../compiler.mjs';
import type { InitOptions } from '../options.init.mjs';
import type { TypstRenderer } from '../renderer.mjs';
import type { RenderToCanvasOptions, RenderSvgOptions } from '../options.render.mjs';

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
 * // optional renderer instance
 * const renderer = enableRendering ?? (() => {
 *   return createGlobalRenderer(createTypstRenderer, pdfJsLib, initOptions);
 * });
 * const $typst = new TypstSnippet(() => {
 *   return createGlobalCompiler(createTypstCompiler, initOptions);
 * }, renderer);
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
   * @param ex the compiler instance, see {@link PromiseJust} and {@link TypstRenderer}.
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
  }

  /** @internal */
  static ccOptions: Partial<InitOptions> | undefined = undefined;
  /**
   * Set compiler init options for initializing global instance {@link $typst}.
   * See {@link InitOptions}.
   */
  setCompilerInitOptions(options: Partial<InitOptions>) {
    if (typeof this.cc !== 'function') {
      throw new Error('compiler has been initialized');
    }
    if (this !== $typst) {
      throw new Error('can not set options for non-global instance');
    }
    TypstSnippet.ccOptions = options;
  }

  /** @internal */
  static exOptions: Partial<InitOptions> | undefined = undefined;
  /**
   * Set renderer init options for initializing global instance {@link $typst}.
   * See {@link InitOptions}.
   */
  setRendererInitOptions(options: Partial<InitOptions>) {
    if (typeof this.ex !== 'function') {
      throw new Error('renderer has been initialized');
    }
    if (this !== $typst) {
      throw new Error('can not set options for non-global instance');
    }
    TypstSnippet.exOptions = options;
  }

  /** @internal */
  static pdfjsModule: unknown | undefined = undefined;
  /**
   * Set pdf.js module for initializing global instance {@link $typst}.
   */
  setPdfjsModule(module: unknown) {
    if (typeof this.ex !== 'function') {
      throw new Error('renderer has been initialized');
    }
    if (this !== $typst) {
      throw new Error('can not set pdfjs module for non-global instance');
    }
    TypstSnippet.pdfjsModule = module;
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
    const rr = await this.getRenderer();
    if (!rr) {
      throw new Error('does not provide renderer instance');
    }
    const data = await this.getVector(o);
    return await rr.runWithSession(async session => {
      rr.manipulateData({
        renderSession: session,
        action: 'reset',
        data,
      });
      return rr.renderSvgDiff({
        ...o,
        renderSession: session,
      });
    });
  }

  /**
   * Compile the document to canvas operations.
   * See {@link SweetRenderOptions} and {@link RenderToCanvasOptions}.
   */
  async canvas(
    container: HTMLElement,
    o?: SweetRenderOptions & Omit<RenderToCanvasOptions, 'container'>,
  ) {
    const rr = await this.getRenderer();
    if (!rr) {
      throw new Error('does not provide renderer instance');
    }
    const data = await this.getVector(o);
    return await rr.runWithSession(async session => {
      rr.manipulateData({
        renderSession: session,
        action: 'reset',
        data,
      });
      rr.renderToCanvas({
        container,
        ...o,
        renderSession: session,
      });
    });
  }

  private async getCompiler() {
    return (typeof this.cc === 'function' ? (this.cc = await this.cc()) : this.cc)!;
  }

  private async getRenderer() {
    return typeof this.ex === 'function' ? (this.ex = await this.ex()) : this.ex;
  }

  private async getCompileOptions(opts?: SweetCompileOptions): Promise<CompileOptions> {
    if (opts === undefined) {
      return { mainFilePath: this.mainFilePath };
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
}

/**
 * The lazy initialized global shared instance of {@link TypstSnippet}. See
 * {@link TypstSnippet} for more details.
 */
export const $typst = new TypstSnippet({
  compiler: async () => {
    // lazy import compile module
    const { createGlobalCompiler } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/contrib/global-compiler.mjs'
    )) as any as typeof import('./global-compiler.mjs');
    const { createTypstCompiler } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/compiler.mjs'
    )) as any as typeof import('../compiler.mjs');

    return createGlobalCompiler(createTypstCompiler, TypstSnippet.ccOptions);
  },
  renderer: async () => {
    // lazy import renderer module
    const { createGlobalRenderer } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs'
    )) as any as typeof import('./global-renderer.mjs');
    const { createTypstRenderer } = (await import(
      '@myriaddreamin/typst.ts/dist/esm/renderer.mjs'
    )) as any as typeof import('../renderer.mjs');

    const pdfjs =
      TypstSnippet.pdfjsModule || (typeof window !== 'undefined' && (window as any)?.pdfjsLib);
    return createGlobalRenderer(createTypstRenderer, pdfjs, TypstSnippet.exOptions);
  },
});
