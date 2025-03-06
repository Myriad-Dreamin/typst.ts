import { CompileArgs, NodeCompiler, ProjectWatcher } from '@myriaddreamin/typst-ts-node-compiler';
import { CompileProvider, OnCompileCallback, TypstHTMLCompiler } from '../compiler.js';
import { ResolvedTypstInput } from '../input.js';

export class NodeCompileProvider extends CompileProvider<NodeCompileProvider> {
  constructor(
    public isWatch: boolean,
    compileArgs: CompileArgs,
    onCompile: OnCompileCallback<NodeCompileProvider>,
    inputRoot?: string,
  ) {
    super(onCompile, compileArgs, inputRoot);
  }

  /**
   * Lazily created compiler.
   */
  compiler = (): NodeCompiler => (this._compiler ||= NodeCompiler.create(this.compileArgs));
  /**
   * Lazily created watcher
   */
  watcher = (): ProjectWatcher => (this._watcher ||= ProjectWatcher.create(this.compileArgs));

  /** @internal */
  /** @internal */
  private _compiler: NodeCompiler | undefined = undefined;
  /** @internal */
  private _watcher: ProjectWatcher | undefined = undefined;

  /**
   * Compiles the source file to the destination file.
   *
   * @param {ResolvedTypstInput} input The resolved input
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  compile = (input: ResolvedTypstInput) => {
    return (project: TypstHTMLCompiler) => {
      this.onCompile(input, project, this);

      // Evicts the cache unused in last 30 runs
      if (this.isWatch) {
        this.watcher().evictCache(30);
      } else {
        this.compiler().evictCache(30);
      }
    };
  };

  /**
   * User trigger compiles the source file to the destination file or watches the source file.
   *
   * All the errors are caught and printed to the console.
   *
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
