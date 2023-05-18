// @ts-ignore
import typstInit, * as typst from '../../compiler/pkg/typst_ts_web_compiler';
import { buildComponent, globalFontPromises } from './init';

import type { InitOptions, BeforeBuildMark } from './options.init';
import { LazyWasmModule } from './wasm';

/**
 * The interface of Typst compiler.
 * @typedef {Object} TypstCompiler
 * @property {function} init - Initialize the Typst compiler.
 * @property {function} compile - Compile a Typst document.
 */
export interface TypstCompiler {
  init(options?: Partial<InitOptions>): Promise<void>;
  reset(): Promise<void>;
  addSource(path: string, source: string, isMain: boolean): Promise<void>;
  getAst(): Promise<string>;
  compile(options: any): Promise<void>;
}

const gCompilerModule = new LazyWasmModule(typstInit);

/**
 * create a Typst compiler.
 * @returns {TypstCompiler} - The Typst compiler.
 * @example
 * ```typescript
 * import { createTypstCompiler } from 'typst';
 * import * as pdfjs from 'pdfjs-dist';
 * const compiler = createTypstCompiler(pdfjs);
 * await compiler.init();
 * await compiler.compile({
 * });
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

  async runSyncCodeUntilStable<T>(execute: () => T): Promise<T> {
    do {
      console.log(this.compiler.get_loaded_fonts());
      const result = await execute();
      console.log(this.compiler.get_loaded_fonts());
      if (globalFontPromises.length > 0) {
        const promises = globalFontPromises.splice(0, globalFontPromises.length);
        const callbacks = await Promise.all(promises);
        for (const callback of callbacks) {
          this.compiler.modify_font_data(callback.idx, new Uint8Array(callback.buffer));
        }
        this.compiler.rebuild();
        continue;
      }
      return result;
    } while (true);
  }

  async reset(): Promise<void> {
    return this.compiler.reset();
  }

  async addSource(path: string, source: string, isMain: boolean): Promise<void> {
    this.compiler.add_source(path, source, isMain);
  }

  async getAst(): Promise<string> {
    return this.runSyncCodeUntilStable(() => this.compiler.get_ast());
  }

  async compile(options: any): Promise<void> {
    console.log('typst compile', options);
  }
}
