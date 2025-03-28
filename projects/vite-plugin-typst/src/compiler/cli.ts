import type {
  CompileArgs,
  CompileDocArgs,
  NodeTypstDocument,
  QueryDocArgs,
} from '@myriaddreamin/typst-ts-node-compiler';
import { spawnSync } from 'child_process';
import { DOMParser, XMLSerializer } from 'xmldom';
import {
  CompileProvider,
  HtmlOutput,
  HtmlOutputExecResult,
  TypstHTMLCompiler,
  TypstHTMLWatcher,
} from '../compiler.js';
import { ResolvedTypstInput } from '../input.js';

class CliHtmlOutput implements HtmlOutput {
  constructor(
    private inner: Document,
    private innerRaw: string,
  ) {}
  private findMeta(name: string) {
    return (
      Array.from(this.inner.getElementsByTagName('meta'))
        .find(e => e.getAttribute('name') === name)
        ?.getAttribute('content') ?? null
    );
  }
  /** Gets the title of the document. */
  title(): string | null {
    return this.findMeta('title');
  }
  /** Gets the description of the document. */
  description(): string | null {
    return this.findMeta('description');
  }
  /** Gets the body of the document. */
  body(): string {
    return new XMLSerializer().serializeToString(this.inner.getElementsByTagName('body')[0]);
  }
  /** Gets the body of the document as bytes. */
  bodyBytes(): Buffer {
    return Buffer.from(this.body());
  }
  /** Gets the HTML of the document. */
  html(): string {
    return this.innerRaw;
  }
  /** Gets the HTML of the document as bytes. */
  htmlBytes(): Buffer {
    return Buffer.from(this.html());
  }
}

class CliHtmlOutputExecResult implements HtmlOutputExecResult {
  constructor(private inner: CliHtmlOutput | { error: string }) {}
  static fromHtml(html: Document, raw: string): CliHtmlOutputExecResult {
    return new CliHtmlOutputExecResult(new CliHtmlOutput(html, raw));
  }
  get result(): CliHtmlOutput | null {
    return this.inner instanceof CliHtmlOutput ? this.inner : null;
  }
  hasError(): boolean {
    return !(this.inner instanceof CliHtmlOutput);
  }
  printErrors(): void {
    this.printDiagnostics();
  }
  printDiagnostics(): void {
    'error' in this.inner && console.error(this.inner.error);
  }
}

class CliWatcher implements TypstHTMLWatcher {
  constructor(private compiler: CliCompiler) {}

  static create(compileArgs: CompileArgs): CliWatcher {
    return new CliWatcher(CliCompiler.create(compileArgs));
  }

  add(paths: string[], exec: (project: TypstHTMLCompiler) => void) {
    exec(this.compiler);
  }

  clear() {}
  watch() {}
}

class CliCompiler implements TypstHTMLCompiler {
  private inputs: Record<string, string> = {};
  private fontArgs: Array<string> = [];
  private rootArgs: Array<string> = [];
  // TODO: version this
  needFeature: boolean = true;
  constructor(private args: CompileArgs = {}) {
    this.inputs = { ...this.inputs, ...(args.inputs ?? {}) };
    this.rootArgs = args.workspace ? ['--root', args.workspace] : [];
    this.fontArgs = args.fontArgs
      ? [
          '--fonts',
          ...args.fontArgs
            .map(it => {
              if ('fontPaths' in it) {
                return it.fontPaths;
              } else {
                throw new Error('Not implemented');
              }
            })
            .reduce((a, b) => a.concat(b), []),
        ]
      : [];
  }
  get featureArgs(): Array<string> {
    return this.needFeature ? ['--features', 'html'] : [];
  }
  static create(args?: CompileArgs): CliCompiler {
    return new CliCompiler(args);
  }
  /** Queries the data of the document. */
  query(compiledOrBy: NodeTypstDocument | CompileDocArgs, args: QueryDocArgs): any {
    if (!('mainFilePath' in compiledOrBy) || !compiledOrBy.mainFilePath) {
      throw new Error('Not implemented');
    }
    let inputs = compiledOrBy.inputs
      ? [
          '--input',
          ...Object.entries({ ...this.inputs, ...compiledOrBy.inputs }).map(
            ([k, v]) => `${k}=${v}`,
          ),
        ]
      : [];
    const result = spawnSync('typst', [
      'query',
      '--format',
      'json',
      ...this.featureArgs,
      ...this.rootArgs,
      compiledOrBy.mainFilePath,
      args.selector,
      ...inputs,
    ]);
    if (result.error) {
      throw new Error(result.error.message);
    }
    return JSON.parse(result.stdout.toString()).map((x: any) => (args.field ? x[args.field] : x));
  }
  /** Compiles the document as a HTML. */
  tryHtml(compiledOrBy: NodeTypstDocument | CompileDocArgs): CliHtmlOutputExecResult {
    if (!('mainFilePath' in compiledOrBy) || !compiledOrBy.mainFilePath) {
      throw new Error('Not implemented');
    }
    let inputs = compiledOrBy.inputs
      ? [
          '--input',
          ...Object.entries({ ...this.inputs, ...compiledOrBy.inputs }).map(
            ([k, v]) => `${k}=${v}`,
          ),
        ]
      : [];
    const result = spawnSync('typst', [
      'compile',
      ...this.featureArgs,
      ...this.rootArgs,
      compiledOrBy.mainFilePath,
      '-',
      '--format',
      'html',
      ...inputs,
    ]);
    if (result.error) {
      return new CliHtmlOutputExecResult({ error: result.error.message });
    }

    const rawRes = result.stdout.toString();
    const parseResult = new DOMParser().parseFromString(rawRes, 'text/html');
    if (!parseResult) {
      return new CliHtmlOutputExecResult({
        error: 'Failed to parse the result\n[stderr]:\n' + result.stderr,
      });
    }
    return CliHtmlOutputExecResult.fromHtml(parseResult, rawRes);
  }
}

export class CliCompileProvider extends CompileProvider {
  OutputType: CliHtmlOutput = undefined!;
  ProjectType: CliCompiler = undefined!;

  /**
   * Lazily created compiler.
   */
  compiler = (): CliCompiler => (this._compiler ||= CliCompiler.create(this.compileArgs));
  /**
   * Lazily created watcher
   */
  watcher = (): CliWatcher => (this._watcher ||= CliWatcher.create(this.compileArgs));

  /**
   * Common getter for the compiler or watcher.
   */
  compilerOrWatcher = () => (this._compiler || this._watcher) ?? null;

  /** @internal */
  private _compiler: CliCompiler | undefined = undefined;
  /** @internal */
  private _watcher: CliWatcher | undefined = undefined;

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
    return (project: TypstHTMLCompiler) => {
      this.onCompile(input, project, this);
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
