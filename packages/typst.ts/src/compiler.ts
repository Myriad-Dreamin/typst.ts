// @ts-ignore
import typstInit, * as typst from '@myriaddreamin/typst-ts-web-compiler';
import { buildComponent, globalFontPromises } from './init';
import { DocumentReference, FsAccessModel } from './internal.types';

import type { InitOptions } from './options.init';
import { RenderPageResult } from './renderer';
import { LazyWasmModule } from './wasm';

export interface CompileOptions {
  mainFilePath: string;
}

/**
 * The interface of Typst compiler.
 * @typedef {Object} TypstCompiler
 * @property {function} init - Initialize the Typst compiler.
 * @property {function} compile - Compile a Typst document.
 */
export interface TypstCompiler {
  /**
   *
   * @param options
   */
  init(options?: Partial<InitOptions>): Promise<void>;
  reset(): Promise<void>;

  addSource(path: string, source: string, isMain: boolean): Promise<void>;

  getAst(mainFilePath: string): Promise<string>;
  compile(options: CompileOptions): Promise<Uint8Array>;

  loadSnapshot(snapshot: unknown, fontServer: FsAccessModel): Promise<any>;
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

  async addSource(path: string, source: string, isMain: boolean): Promise<void> {
    await new Promise<void>(resolve => {
      this.compiler.add_source(path, source, isMain);
      resolve(undefined);
    });
  }

  async getAst(mainFilePath: string): Promise<string> {
    return this.runSyncCodeUntilStable(() => this.compiler.get_ast(mainFilePath));
  }

  compile(options: CompileOptions): Promise<Uint8Array> {
    return new Promise<Uint8Array>(resolve => {
      resolve(this.compiler.compile(options.mainFilePath));
    });
  }

  renderPageToCanvas(): Promise<any> {
    throw new Error('Please use the api TypstRenderer.renderToCanvas in v0.4.0');
  }
}
