// @ts-ignore
import typstInit, * as typst from '@myriaddreamin/typst-ts-web-compiler';
import { buildComponent, globalFontPromises } from './init';
import { FsAccessModel } from './internal.types';

import type { InitOptions } from './options.init';
import { RenderPageResult } from './renderer';
import { LazyWasmModule } from './wasm';

export interface CompileOptions {
  mainFilePath: string;
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
  compile(options: CompileOptions): Promise<Uint8Array>;

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
   */
  loadSnapshot(snapshot: unknown, fontServer: FsAccessModel): Promise<any>;
}

const gCompilerModule = new LazyWasmModule(typstInit);

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

class TypstCompilerDriver {
  compiler: typst.TypstCompiler;

  constructor() {}

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.compiler = await buildComponent(options, gCompilerModule, typst.TypstCompilerBuilder, {});
  }

  compile(options: CompileOptions): Promise<Uint8Array> {
    return new Promise<Uint8Array>(resolve => {
      resolve(this.compiler.compile(options.mainFilePath));
    });
  }

  async getAst(mainFilePath: string): Promise<string> {
    return this.runSyncCodeUntilStable(() => this.compiler.get_ast(mainFilePath));
  }

  async runSyncCodeUntilStable<T>(execute: () => T): Promise<T> {
    for (;;) {
      console.log(this.compiler.get_loaded_fonts());
      const result = execute();
      console.log(this.compiler.get_loaded_fonts());
      if (globalFontPromises.length > 0) {
        const promises = Promise.all(globalFontPromises.splice(0, globalFontPromises.length));
        const callbacks: {
          buffer: ArrayBuffer;
          idx: number;
        }[] = await promises;
        for (const callback of callbacks) {
          this.compiler.modify_font_data(callback.idx, new Uint8Array(callback.buffer));
        }
        this.compiler.rebuild();
        continue;
      }
      return result;
    }
  }

  async reset(): Promise<void> {
    await new Promise<void>(resolve => {
      this.compiler.reset();
      resolve(undefined);
    });
  }

  loadSnapshot(snapshot: unknown, fontServer: FsAccessModel): Promise<void> {
    return new Promise<any>(resolve => {
      resolve(this.compiler.load_snapshot(snapshot, (p: string) => fontServer.readAll(p)));
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
