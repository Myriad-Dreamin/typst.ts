export type CompilerBackend = 'auto' | 'wasm' | 'wasm-worker' | 'node' | 'cli';

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
  fontProvider?: FontProvider | null;
  accessModel?: unknown | null;
  packageProvider?: unknown | null;
  packageRegistry?: unknown | null;
}

export interface WorkerLike {
  postMessage(message: unknown, transfer?: readonly unknown[]): void;
  terminate?(): void | Promise<number | void>;
  close?(): void;
  addEventListener?(type: 'message' | 'error', listener: (event: unknown) => void): void;
  removeEventListener?(type: 'message' | 'error', listener: (event: unknown) => void): void;
  on?(type: 'message' | 'error' | 'exit', listener: (...args: any[]) => void): void;
  off?(type: 'message' | 'error' | 'exit', listener: (...args: any[]) => void): void;
  removeListener?(type: 'message' | 'error' | 'exit', listener: (...args: any[]) => void): void;
}

export interface WasmWorkerBackendOptions {
  worker?: WorkerLike;
  workerFactory?: () => WorkerLike | Promise<WorkerLike>;
  workerUrl?: string | { href: string };
  workerOptions?: Record<string, unknown>;
  wasm?: WasmBackendOptions;
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
  wasmWorker?: WasmWorkerBackendOptions;
  node?: Record<string, unknown>;
  cli?: CliBackendOptions;
  initOptions?: unknown;
  fontProvider?: FontProvider | null;
  accessModel?: unknown | null;
  packageProvider?: unknown | null;
  packageRegistry?: unknown | null;
}

export type FontInput =
  | string
  | Uint8Array
  | {
    info: unknown;
    blob?: (index: number) => Uint8Array;
    url?: string;
  };

export type FontProvider =
  | FontInput[]
  | {
    fonts?: FontInput | FontInput[];
    rawFonts?: Uint8Array | Uint8Array[];
    fontData?: Uint8Array | Uint8Array[];
    lazyFonts?: FontInput | FontInput[];
    /** Advanced low-level hooks passed to @myriaddreamin/typst.ts init beforeBuild. */
    beforeBuild?: unknown | unknown[];
    /** Options forwarded to @myriaddreamin/typst.ts loadFonts. */
    loadOptions?: unknown;
    /** Deprecated compatibility alias for loadOptions. */
    options?: unknown;
  };

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
  setFontProvider?(provider: FontProvider | null): Promise<void>;
  setAccessModel?(accessModel: unknown | null): Promise<void>;
  setPackageProvider?(packageProvider: unknown | null): Promise<void>;
  setPackageRegistry?(packageRegistry: unknown | null): Promise<void>;
  terminate?(): void | Promise<void>;
}

export function createCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createNodeCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createWasmCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createWasmWorkerCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
export function createCliCompiler(options?: CreateCompilerOptions): Promise<TypstCompilerFacade>;
