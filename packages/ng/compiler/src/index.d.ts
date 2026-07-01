export type CompilerBackend = 'auto' | 'wasm' | 'node' | 'cli';

export interface CompileInput {
  mainFilePath?: string;
  mainFileContent?: string;
  /** Deprecated compatibility alias for browser snippet-style inputs. */
  mainContent?: string;
  workspace?: string;
  root?: string;
  inputs?: Record<string, string>;
  diagnostics?: 'none' | 'unix' | 'full';
  resetRead?: boolean;
}

export interface WasmBackendOptions {
  initOptions?: unknown;
}

export interface CliBackendOptions {
  /** Official Typst CLI command. Used for pdf/svg/html exports. */
  command?: string;
  /** typst.ts CLI command. Used for vector artifacts. */
  vectorCommand?: string;
  /** Deprecated alias for vectorCommand. */
  typstTsCommand?: string;
  cwd?: string;
  env?: Record<string, string | undefined>;
}

export interface CreateCompilerOptions {
  backend?: CompilerBackend;
  wasm?: WasmBackendOptions;
  node?: Record<string, unknown>;
  cli?: CliBackendOptions;
  initOptions?: unknown;
}

export interface TypstCompilerFacade<CompiledDocument = unknown> {
  readonly backend: Exclude<CompilerBackend, 'auto'>;
  compile(input: CompileInput): Promise<CompiledDocument | Uint8Array>;
  vector(input: CompileInput | CompiledDocument): Promise<Uint8Array>;
  pdf(input: CompileInput | CompiledDocument, options?: unknown): Promise<Uint8Array>;
  plainSvg(input: CompileInput | CompiledDocument): Promise<string>;
  svg(input: CompileInput | CompiledDocument): Promise<string>;
  html(input: CompileInput | CompiledDocument): Promise<string | null>;
  query?(input: CompileInput | CompiledDocument, options: unknown): Promise<unknown>;
  addSource?(path: string, source: string): void | Promise<void>;
  mapShadow?(path: string, content: Uint8Array): void | Promise<void>;
  unmapShadow?(path: string): void | Promise<void>;
  resetShadow?(): void | Promise<void>;
  reset?(): void | Promise<void>;
  evictCache?(maxAge?: number): void | Promise<void>;
}

export function createCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createNodeCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createWasmCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createCliCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
