import * as path from 'path';
import { NodeCompiler, ProjectWatcher, CompileArgs } from '@myriaddreamin/typst-ts-node-compiler';
import type { NodeTypstProject } from '@myriaddreamin/typst-ts-node-compiler';
import type { TypstPluginOptions } from '.';

/**
 * The callback to be called when the document is compiled.
 *
 * @param source The source file path
 * @param result The compiled project
 * @param ctx The compile provider
 */
export type OnCompileCallback = (
  source: string,
  result: NodeTypstProject,
  ctx: NodeCompileProvider,
) => void;
export type CompileProvider = NodeCompileProvider;

/**
 * Creates a new provider for the plugin.
 *
 * @param options The plugin options
 * @param onCompile The callback to be called when the document is compiled
 * @returns
 */
export const makeProvider = (options: TypstPluginOptions, onCompile: OnCompileCallback) => {
  const compileArgs: CompileArgs = {
    workspace: options.root || '.',
    inputs: {
      'x-target': 'web-light',
      // ...(urlBase ? { 'x-url-base': urlBase } : {}),
    },
    fontArgs: [{ fontPaths: ['./assets/fonts', './assets/typst-fonts'] }],
  };

  const compilerProvider = options?.compiler || '@myriaddreamin/typst-ts-node-compiler';
  if (compilerProvider !== '@myriaddreamin/typst-ts-node-compiler') {
    throw new Error(`Unsupported compiler provider: ${compilerProvider}`);
  }

  return new NodeCompileProvider(compileArgs, false, onCompile);
};

class NodeCompileProvider {
  compiled = new Map<string, string>();

  constructor(
    compileArgs: CompileArgs,
    public isWatch: boolean,
    onCompile: OnCompileCallback,
  ) {
    this.compileArgs = compileArgs;
    this.onCompile = onCompile;
  }

  resolveRel(input: string, ext = '.html') {
    const rel = input.endsWith('.typ') ? input.slice(0, -4) : input;
    return path.relative(this.inputRoot, rel + ext);
  }

  /**
   * Lazily created compiler.
   */
  compiler = (): NodeCompiler => (this._compiler ||= NodeCompiler.create(this.compileArgs));
  /**
   * Lazily created watcher
   */
  watcher = (): ProjectWatcher => (this._watcher ||= ProjectWatcher.create(this.compileArgs));

  /**
   * Common getter for the compiler or watcher.
   */
  compilerOrWatcher = () => this._compiler || this._watcher;

  /** @internal */
  inputRoot: string = '.';
  /** @internal */
  onCompile: OnCompileCallback;
  /** @internal */
  readonly compileArgs: CompileArgs;
  /** @internal */
  private _compiler: NodeCompiler | undefined = undefined;
  /** @internal */
  private _watcher: ProjectWatcher | undefined = undefined;

  /**
   * Compiles the source file to the destination file.
   *
   * @param {string} src The source file path
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  compile = (src: string) => {
    return (project: NodeTypstProject) => {
      this.onCompile(src, project, this);

      // Evicts the cache unused in last 30 runs
      this.compilerOrWatcher()?.evictCache(30);
    };
  };

  /**
   * User trigger compiles the source file to the destination file or watches the source file.
   *
   * All the errors are caught and printed to the console.
   *
   * @param {string} src The source file path
   *
   * @example
   * compileOrWatch("src/index.typ", "dist/index.html");
   */
  compileOrWatch = (src: string) => {
    try {
      if (this.isWatch) {
        this.watcher().add([src], this.compile(src));
      } else {
        this.compile(src)(this.compiler());
      }
    } catch (e) {
      console.error(e);
      return;
    }
  };
}
