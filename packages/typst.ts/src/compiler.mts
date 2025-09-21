// @ts-ignore
import type * as typst from '@myriaddreamin/typst-ts-web-compiler';
import { buildComponent } from './init.mjs';
import { SemanticTokens, SemanticTokensLegend, kObject } from './internal.types.mjs';

import { loadFonts, type InitOptions } from './options.init.mjs';
import { LazyWasmModule } from './wasm.mjs';

/**
 * Available formats for compiling the document.
 */
export type CompileFormat = 'vector' | 'pdf';

/**
 * The diagnostic message partially following the LSP specification.
 */
interface DiagnosticMessage {
  // The package owning the path.
  // If the package is empty, the path is a relative path to the *workspace root*.
  package: string;
  // The path of the file.
  path: string;
  // Severity of the diagnostic message.
  severity: string;
  // Zero-based line number and one-based character offset.
  // The range of the diagnostic message.
  // If the diagnostic message is a range, the range is in the format of `startLine:startCharacter-endLine:endCharacter`.
  // If the diagnostic message is a point, the range is in the format of `line:character`.
  // Otherwise, the range is empty.
  range: string;
  // The message of the diagnostic message.
  message: string;
}

/**
 * Available formats for compiling the document.
 *
 * If set to unix, a diagnostics is in format of
 *
 * ```log
 * // with package
 * cetz:0.2.0@lib.typ:2:9-3:15: error: unexpected type in `+` application
 * // without package
 * main.typ:2:9-3:15: error: unexpected type in `+` application
 * ```
 *
 * If set to long, a diagnostics is in format of {@link DiagnosticMessage}.
 *
 * If set to full, a diagnostics is in format of {@link DiagnosticMessage}, but also with trace messages.
 */
export type DiagnosticsFormat = 'none' | 'unix' | 'full';

export type DiagnosticsData = {
  none: never;
  unix: string;
  full: DiagnosticMessage;
};

interface DiagOpts<D extends DiagnosticsFormat = DiagnosticsFormat> {
  /**
   * Whether to include diagnostic information in the result.
   * Note: it will be set to 'full' by default in v0.6.0
   * @default 'full'
   */
  diagnostics?: D;
}

interface SnapshotOptions {
  /**
   * The path of the main file.
   */
  mainFilePath: string;
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

interface TransientCompileOptions<
  F extends CompileFormat = any,
  Diagnostics extends DiagnosticsFormat = DiagnosticsFormat,
> extends SnapshotOptions,
  DiagOpts<Diagnostics> {
  /**
   * The format of the artifact.
   * - 'vector': can then load to the renderer to render the document.
   * - 'pdf': for finally exporting pdf to the user.
   * @default 'vector'
   */
  format?: F;
}

interface IncrementalCompileOptions<Diagnostics extends DiagnosticsFormat = DiagnosticsFormat>
  extends SnapshotOptions,
  DiagOpts<Diagnostics> {
  /**
   * The format of the incrementally exported artifact.
   * @default 'vector'
   */
  format?: 'vector';
  /**
   * The incremental server for the document.
   */
  incrementalServer: IncrementalServer;
}

export interface QueryOptions {
  /**
   * select part of document for query.
   */
  selector: string;
  /**
   * cast result by accessing single field.
   */
  field?: string;
}

/**
 * The options for compiling the document.
 */
export type CompileOptions<
  Format extends CompileFormat = any,
  Diagnostics extends DiagnosticsFormat = DiagnosticsFormat,
> = TransientCompileOptions<Format, Diagnostics> | IncrementalCompileOptions;

export class IncrementalServer {
  /**
   * @internal
   */
  [kObject]: typst.IncrServer;

  /**
   * @internal
   */
  constructor(s: typst.IncrServer) {
    this[kObject] = s;
  }

  /**
   * Reset the incremental server to the initial state.
   */
  reset(): void {
    this[kObject].reset();
  }

  /**
   * Return current result.
   */
  current(): Uint8Array | undefined {
    return this[kObject].current();
  }

  /**
   * Also attach the debug info to the result.
   */
  setAttachDebugInfo(enable: boolean): void {
    this[kObject].set_attach_debug_info(enable);
  }
}

interface CompileResult<T, D extends DiagnosticsFormat> {
  result?: T;
  diagnostics?: DiagnosticsData[D][];
}

export interface TypstFontInfo { }

enum TypstFontResolverCons { }
export type TypstFontResolver = TypstFontResolverCons;

export interface TypstFontBuilder {
  /**
   * Initialize the font builder.
   * @param options - The options for initializing the font builder.
   */
  init(options?: Partial<InitOptions>): Promise<void>;
  /**
   * Get the font info.
   *
   * @param font_buffer - The font buffer to get the font info.
   * @returns {TypstFontInfo} - The font info.
   */
  getFontInfo(font_buffer: Uint8Array): Promise<TypstFontInfo>;
  /**
   * Add a raw font.
   *
   * @param font_buffer - The font buffer to add.
   */
  addFontData(font_buffer: Uint8Array): Promise<void>;
  /**
   * Add a lazy font.
   *
   * @param info - The font info, usually from {@link getFontInfo}.
   * @param blob - The blob function to get the font buffer.
   * @param context - The context.
   */
  addLazyFont(
    info: TypstFontInfo,
    blob: (idx: number) => Uint8Array,
    context?: object,
  ): Promise<void>;

  /**
   * Build the font resolver. The font resolver will be freed after the callback
   * is invoked and before returning the build function.
   *
   * @param cb - The function to use the font resolver.
   * @returns {Promise<T>} - The result of the function.
   */
  build<T>(cb: (resolver: TypstFontResolver) => Promise<T>): Promise<T>;
}

/**
 * create a Typst font builder.
 * @returns {TypstFontBuilder} - The Typst font builder.
 * @example
 * ```typescript
 * import { createTypstFontBuilder } from 'typst';
 * const fb = createTypstFontBuilder();
 * await fb.init();
 * await fb.addFontData(new Uint8Array(await fetch('font.ttf').then(r => r.arrayBuffer())));
 * await fb.build();
 * ```
 */
export function createTypstFontBuilder(): TypstFontBuilder {
  return new TypstFontBuilderDriver();
}

export class TypstWorld {
  private [kObject]: typst.TypstCompileWorld;

  constructor(world: typst.TypstCompileWorld) {
    this[kObject] = world;
  }

  /**
   * Compile the paged document.
   *
   * @param {DiagnosticsFormat} format - The format of the diagnostics.
   * @returns {Promise<{ diagnostics?: DiagnosticsData[DiagnosticsFormat][] }>} - The result of the compilation.
   */
  compile<D extends DiagnosticsFormat = 'full'>(
    opts?: DiagOpts<D>,
  ): Promise<CompileResult<undefined, D> & { hasError: boolean }> {
    return this[kObject].compile(0, getDiagnosticsArg(opts?.diagnostics));
  }

  /**
   * Compile the paged document.
   *
   * @param {DiagnosticsFormat} format - The format of the diagnostics.
   * @returns {Promise<{ diagnostics?: DiagnosticsData[DiagnosticsFormat][] }>} - The result of the compilation.
   */
  compileHtml<D extends DiagnosticsFormat = 'full'>(
    opts?: DiagOpts<D>,
  ): Promise<CompileResult<undefined, D> & { hasError: boolean }> {
    return this[kObject].compile(1, getDiagnosticsArg(opts?.diagnostics));
  }

  /**
   * Runs query on the paged document.
   */
  async query(options: QueryOptions): Promise<string> {
    return this[kObject].query(0, options.selector, options.field);
  }

  /**
   * Get the title of the paged document.
   * Throw error if the world didn't compile the paged document.
   *
   * @returns {string | undefined} - The title of the paged document.
   */
  title(): string | undefined {
    return this[kObject].title(0);
  }

  /**
   * Export the paged document as vector format.
   *
   * @returns {Uint8Array | undefined} - The title of the paged document.
   */
  vector<D extends DiagnosticsFormat = 'full'>(
    opts?: DiagOpts<D>,
  ): Promise<CompileResult<Uint8Array, D>> {
    return this[kObject].get_artifact(0, getDiagnosticsArg(opts?.diagnostics)) || {};
  }

  /**
   * Export the paged document to PDF.
   *
   * @returns {Uint8Array | undefined} - The title of the paged document.
   */
  pdf<D extends DiagnosticsFormat = 'full'>(
    opts?: DiagOpts<D>,
  ): Promise<CompileResult<Uint8Array, D>> {
    return this[kObject].get_artifact(1, getDiagnosticsArg(opts?.diagnostics)) || {};
  }
}

/**
 * The interface of Typst compiler.
 */
export interface TypstCompiler {
  /**
   * Initialize the typst compiler.
   * @param {Partial<InitOptions>} options - The options for initializing the
   * typst compiler.
   */
  init(options?: Partial<InitOptions>): Promise<void>;

  /**
   * Reset the typst compiler to the initial state.
   * Note: without calling this function, the compiler will always keep caches
   * such as:
   * - loaded fonts
   * - source files corresponding to typst modules
   *
   * Note: this function is independent to the {@link resetShadow} function.
   * This is intended to optimize the performance of the compiler.
   */
  reset(): Promise<void>;

  /**
   * Compile an document with the maintained state.
   * @param {CompileOptions} options - The options for compiling the document.
   * @returns {Promise<Uint8Array>} - artifact in vector format.
   * You can then load the artifact to the renderer to render the document.
   */
  compile<D extends DiagnosticsFormat>(
    options: CompileOptions<'vector', D>,
  ): Promise<CompileResult<Uint8Array, D>>;
  compile<D extends DiagnosticsFormat>(
    options: CompileOptions<'pdf', D>,
  ): Promise<CompileResult<Uint8Array, D>>;
  compile<D extends DiagnosticsFormat>(
    options: CompileOptions<any, D>,
  ): Promise<CompileResult<Uint8Array, D>>;

  runWithWorld<T>(options: SnapshotOptions, cb: (world: TypstWorld) => Promise<T>): Promise<T>;

  /**
   * Set the fonts to the compiler. Note: multiple compilers can share the same fonts.
   *
   * @param {TypstFontResolver} fonts - The fonts to set.
   */
  setFonts(fonts: TypstFontResolver): void;

  /**
   * experimental
   * Query the result with document
   */
  query<T>(options: QueryOptions & SnapshotOptions): Promise<T>;

  /**
   * Print the AST of the main file.
   * @param {string} mainFilePath - The path of the main file.
   * @returns {Promise<string>} - an string representation of the AST.
   */
  getAst(mainFilePath: string): Promise<string>;

  /**
   * Add a source file to the compiler.
   * @param {string} path - The path of the source file.
   * @param {string} source - The source code of the source file.
   *
   */
  addSource(path: string, source: string): void;

  /**
   * Add a shadow file to the compiler.
   * @param {string} path - The path to the shadow file.
   * @param {Uint8Array} content - The content of the shadow file.
   *
   */
  mapShadow(path: string, content: Uint8Array): void;

  /**
   * Remove a shadow file from the compiler.
   * @param {string} path - The path to the shadow file.
   */
  unmapShadow(path: string): void;

  /**
   * Reset the shadow files.
   * Note: this function is independent to the {@link reset} function.
   */
  resetShadow(): void;

  /**
   * experimental
   * See Semantic tokens: https://github.com/microsoft/vscode/issues/86415
   */
  getSemanticTokenLegend(): Promise<SemanticTokensLegend>;

  /**
   * experimental
   * See Semantic tokens: https://github.com/microsoft/vscode/issues/86415
   *
   * @param {string} opts.mainFilePath - The path of the main file.
   * @param {string} opts.resultId - The id of the result.
   * @param {string} opts.offsetEncoding - The encoding of the offset.
   *   - 'utf-16': the offset is encoded in utf-16.
   *   - 'utf-8': the offset is encoded in utf-8.
   *   @default 'utf-16'
   * @returns {Promise<SemanticTokens>} - The semantic tokens.
   */
  getSemanticTokens(opts: {
    mainFilePath: string;
    resultId?: string;
    offsetEncoding?: string;
  }): Promise<SemanticTokens>;

  /**
   * experimental
   * Run with an incremental server which holds the state of the document in wasm.
   *
   * @param {function(IncrementalServer): Promise<T>} f - The function to run with the incremental server.
   * @returns {Promise<T>} - The result of the function.
   *
   * Note: the incremental server will be freed after the function is finished.
   */
  withIncrementalServer<T>(f: (s: IncrementalServer) => Promise<T>): Promise<T>;
}

const gCompilerModule = new LazyWasmModule(async (bin?: any) => {
  const module = await import('@myriaddreamin/typst-ts-web-compiler');
  return await module.default(bin);
});

/**
 * create a Typst compiler.
 * @returns {TypstCompiler} - The Typst compiler.
 * @example
 * ```typescript
 * import { createTypstCompiler } from 'typst';
 * const compiler = createTypstCompiler();
 * await compiler.init();
 * compiler.addSource('/main.typ', 'Hello, typst!');
 * await compiler.compile({ mainFilePath: '/main.typ' });
 * ```
 */
export function createTypstCompiler(): TypstCompiler {
  return new TypstCompilerDriver();
}

export class TypstFontBuilderDriver implements TypstFontBuilder {
  private fontBuilderJs: typeof typst;
  private fontBuilder: typst.TypstFontResolverBuilder;

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.fontBuilderJs = await import('@myriaddreamin/typst-ts-web-compiler');
    /// init typst wasm module
    await gCompilerModule.init(options?.getModule?.());

    this.fontBuilder = new this.fontBuilderJs.TypstFontResolverBuilder();
  }

  async getFontInfo(font_buffer: Uint8Array): Promise<TypstFontInfo> {
    return this.fontBuilder.get_font_info(font_buffer) as TypstFontInfo;
  }

  async addFontData(font_buffer: Uint8Array): Promise<void> {
    this.fontBuilder.add_raw_font(font_buffer);
  }
  async addLazyFont<C extends TypstFontInfo>(
    info: TypstFontInfo,
    blob: (this: C, idx: number) => Uint8Array,
  ): Promise<void> {
    return this.fontBuilder.add_lazy_font(info, blob);
  }
  async build<T>(cb: (resolver: TypstFontResolver) => Promise<T>): Promise<T> {
    const fonts = await this.fontBuilder.build();
    const result = await cb(fonts as any as TypstFontResolver);
    fonts.free();
    return result;
  }
}

class TypstCompilerDriver implements TypstCompiler {
  compiler: typst.TypstCompiler;
  compilerJs: typeof typst;

  static defaultAssets = ['text' as const];

  constructor() { }

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.compilerJs = await import('@myriaddreamin/typst-ts-web-compiler');
    const TypstCompilerBuilder = this.compilerJs.TypstCompilerBuilder;

    const compilerOptions = { ...(options || {}) };
    const beforeBuild = (compilerOptions.beforeBuild ??= []);
    const hasPreloadRemoteFonts = beforeBuild.some(
      (fn: any) => fn._preloadRemoteFontOptions !== undefined,
    );
    const hasSpecifiedAssets = beforeBuild.some(
      (fn: any) => fn._preloadRemoteFontOptions?.assets !== undefined,
    );
    const hasDisableAssets = beforeBuild.some(
      (fn: any) => fn._preloadRemoteFontOptions?.assets === false,
    );

    if (!hasPreloadRemoteFonts || (!hasSpecifiedAssets && !hasDisableAssets)) {
      beforeBuild.push(loadFonts([], { assets: TypstCompilerDriver.defaultAssets }));
    }

    const hasFontLoader = beforeBuild.some((fn: any) => fn._kind === 'fontLoader');
    if (!hasFontLoader) {
      throw new Error(
        'TypstCompiler: no font loader found, please use font loaders, e.g. loadFonts or preloadSystemFonts',
      );
    }
    this.compiler = await buildComponent(options, gCompilerModule, TypstCompilerBuilder, {});
  }

  setFonts(fonts: TypstFontResolver): void {
    this.compiler.set_fonts(fonts as any as typst.TypstFontResolver);
  }

  compile(options: CompileOptions): Promise<any> {
    return new Promise(resolve => {
      const world = this.compiler.snapshot(
        options.root,
        options.mainFilePath,
        convertInputs(options.inputs),
      );
      if ('incrementalServer' in options) {
        resolve(
          world.incr_compile(
            options.incrementalServer[kObject],
            getDiagnosticsArg(options.diagnostics),
          ),
        );
        return;
      }
      resolve(
        world.get_artifact(options.format || 'vector', getDiagnosticsArg(options.diagnostics)),
      );
    });
  }

  async runWithWorld<T>(
    options: SnapshotOptions,
    cb: (world: TypstWorld) => Promise<T>,
  ): Promise<T> {
    const world = this.compiler.snapshot(
      options.root,
      options.mainFilePath,
      convertInputs(options.inputs),
    );
    let result = await cb(new TypstWorld(world));
    world.free();
    return result;
  }

  query(options: QueryOptions & SnapshotOptions): Promise<any> {
    return this.runWithWorld(options, async world => {
      return JSON.parse(await world.query(options));
    });
  }

  getSemanticTokenLegend(): Promise<SemanticTokensLegend> {
    return new Promise<SemanticTokensLegend>(resolve => {
      resolve(this.compiler.get_semantic_token_legend());
    });
  }

  getSemanticTokens(opts: {
    mainFilePath: string;
    resultId?: string;
    offsetEncoding?: string;
  }): Promise<SemanticTokens> {
    return new Promise<SemanticTokens>(resolve => {
      this.compiler.reset();
      resolve(
        this.compiler.get_semantic_tokens(
          opts.offsetEncoding || 'utf-16',
          opts.mainFilePath,
          opts.resultId,
        ) as any,
      );
    });
  }

  async withIncrementalServer<T>(f: (s: IncrementalServer) => Promise<T>): Promise<T> {
    const srv = new IncrementalServer(this.compiler.create_incr_server());
    try {
      return await f(srv);
    } finally {
      srv[kObject].free();
    }
  }

  async getAst(mainFilePath: string): Promise<string> {
    return this.compiler.get_ast(mainFilePath);
  }

  async reset(): Promise<void> {
    await new Promise<void>(resolve => {
      this.compiler.reset();
      resolve(undefined);
    });
  }

  addSource(path: string, source: string): void {
    if (arguments.length > 2) {
      throw new Error(
        'use of addSource(path, source, isMain) is deprecated, please use addSource(path, source) instead',
      );
    }

    this.compiler.add_source(path, source);
  }

  mapShadow(path: string, content: Uint8Array): void {
    this.compiler.map_shadow(path, content);
  }

  unmapShadow(path: string): void {
    this.compiler.unmap_shadow(path);
  }

  resetShadow(): void {
    this.compiler.reset_shadow();
  }

  renderPageToCanvas(): Promise<any> {
    throw new Error('Please use the api TypstRenderer.renderToCanvas in v0.4.0');
  }
}
createTypstCompiler._impl = TypstCompilerDriver;

// todo: caching inputs
function convertInputs(inputs?: Record<string, string>): [string, string][] | undefined {
  return inputs ? Object.entries(inputs) : undefined;
}

function getDiagnosticsArg(diagnostics: string | undefined): number {
  switch (diagnostics) {
    case 'none':
      return 1;
    case 'unix':
      return 2;
    case 'full':
    default:
      return 3;
  }
}
