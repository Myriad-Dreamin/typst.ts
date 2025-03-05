import { CompileArgs, NodeCompiler, ProjectWatcher, type NodeTypstProject } from '@myriaddreamin/typst-ts-node-compiler';
import * as path from 'path';
import { OnCompileCallback } from '../compiler';
import { ResolvedTypstInput } from '../input';

export class NodeCompileProvider {
  compiled = new Map<string, string>();

  constructor(
    compileArgs: CompileArgs,
    public isWatch: boolean,
    onCompile: OnCompileCallback
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
   * @param {ResolvedTypstInput} input The resolved input
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  compile = (input: ResolvedTypstInput) => {
    return (project: NodeTypstProject) => {
      this.onCompile(input, project, this);

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
   * @param {ResolvedTypstInput} input The resolved input
   *
   * @example
   * compileOrWatch("src/index.typ", "dist/index.html");
   */
  compileOrWatch = (input: ResolvedTypstInput) => {
    try {
      if (this.isWatch) {
        this.watcher().add([input.mainFilePath], this.compile(input));
      } else {
        this.compile(input)(this.compiler());
      }
    } catch (e) {
      console.error(e);
      return;
    }
  };
}
