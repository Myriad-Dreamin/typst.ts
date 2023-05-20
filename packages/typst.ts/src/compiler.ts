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
  getAst(main_file_path: string): Promise<string>;
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
    this.compiler = await buildComponent(options, gCompilerModule, typst.TypstCompilerBuilder, {
      latelyBuild({ builder }: { builder: typst.TypstCompilerBuilder }) {
        builder.set_access_model(
          this,
          // todo: named parameters
          (path: string) => {
            console.log('mtime', path);

            const request = new XMLHttpRequest();
            request.open('HEAD', 'http://localhost:20810/' + path, false);
            request.send(null);

            let lastModified = undefined;
            if (request.status === 200) {
              lastModified = request.getResponseHeader('Last-Modified');
            }
            if (lastModified) {
              lastModified = new Date(lastModified).getTime();
            } else {
              lastModified = 0;
            }
            return lastModified;
          },
          (path: string) => {
            console.log('is file', path);

            const request = new XMLHttpRequest();
            request.open('HEAD', 'http://localhost:20810/' + path, false);
            request.send(null);

            if (request.status === 200) {
              console.log(request, request.getAllResponseHeaders());
            }
            return true;
          },
          (path: string) => {
            console.log('real path', path);
            return path;
          },
          (path: string) => {
            console.log('read all', path);

            const request = new XMLHttpRequest();
            request.overrideMimeType('text/plain; charset=x-user-defined');
            request.open('GET', 'http://localhost:20810/' + path, false);
            request.send(null);

            if (request.status === 200) {
              return Uint8Array.from(request.response, (c: string) => c.charCodeAt(0));
            }
            return undefined;
          },
        );
      },
    });
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

  async getAst(main_file: string): Promise<string> {
    return this.runSyncCodeUntilStable(() => this.compiler.get_ast(main_file));
  }

  async compile(options: any): Promise<void> {
    console.log('typst compile', options);
  }
}
