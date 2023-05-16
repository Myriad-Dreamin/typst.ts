// @ts-ignore
import typstInit, * as typst from '../../compiler/pkg/typst_ts_web_compiler';

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

  async loadFont(builder: typst.TypstCompilerBuilder, fontPath: string): Promise<void> {
    const response = await fetch(fontPath);
    const fontBuffer = new Uint8Array(await response.arrayBuffer());
    await builder.add_raw_font(fontBuffer);
  }

  async reset(): Promise<void> {
    return this.compiler.reset();
  }

  async addSource(path: string, source: string, isMain: boolean): Promise<void> {
    this.compiler.add_source(path, source, isMain);
  }

  async getAst(): Promise<string> {
    return this.compiler.get_ast();
  }

  async init(options?: Partial<InitOptions>): Promise<void> {
    /// init typst wasm module
    if (options?.getModule) {
      await gCompilerModule.init(options.getModule());
    }

    /// build typst compiler
    let builder = new typst.TypstCompilerBuilder();
    const buildCtx = { ref: this, builder };

    for (const fn of options?.beforeBuild ?? []) {
      await fn(undefined as unknown as BeforeBuildMark, buildCtx);
    }
    this.compiler = await builder.build();
  }
  async compile(options: any): Promise<void> {
    console.log('typst compile', options);
  }
}
