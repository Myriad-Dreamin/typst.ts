import {
  CompileArgs,
  CompileDocArgs,
  NodeError,
  NodeTypstCompileResult,
  NodeTypstDocument,
  NodeTypstProject,
  ProjectWatcher,
  QueryDocArgs,
  RenderPdfOpts,
} from '@myriaddreamin/typst-ts-node-compiler';
import { DOMParser, XMLSerializer } from 'xmldom';
import { spawnSync } from 'child_process';
import { OnCompileCallback } from '../compiler';
import path from 'path';
import { ResolvedTypstInput } from '../input';

class CliTypstDocument {
  /** Gets the number of pages in the document. */
  get numOfPages(): number {
    throw new Error('Not implemented');
  }
  /** Gets the title of the document. */
  get title(): string | null {
    throw new Error('Not implemented');
  }
  /** Gets the authors of the document. */
  get authors(): Array<string> | null {
    throw new Error('Not implemented');
  }
  /** Gets the keywords of the document. */
  get keywords(): Array<string> | null {
    throw new Error('Not implemented');
  }
  /**
   * Gets the unix timestamp (in nanoseconds) of the document.
   *
   * Note: currently typst doesn't specify the timezone of the date, and we
   * keep stupid and doesn't add timezone info to the date.
   */
  get date(): number | null {
    throw new Error('Not implemented');
  }
  /**
   * Determines whether the date should be automatically generated.
   *
   * This happens when user specifies `date: auto` in the document
   * explicitly.
   */
  get enabledAutoDate(): boolean {
    throw new Error('Not implemented');
  }
}

class CliHtmlOutput {
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

class CliTypstCompileResult {
  constructor(private inner: CliTypstDocument | { error: string }) {}
  /** Gets the result of execution. */
  get result(): CliTypstDocument | null {
    if (this.hasError()) {
      return null;
    }
    throw new Error('Not implemented');
  }
  /** Takes the result of execution. */
  takeWarnings(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Takes the error of execution. */
  takeError(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Takes the diagnostics of execution. */
  takeDiagnostics(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Whether the execution has error. */
  hasError(): boolean {
    return !(this.inner instanceof CliTypstDocument);
  }
  /** Prints the errors during execution. */
  printErrors(): void {
    'error' in this.inner && console.error(this.inner.error);
  }
  /** Prints the diagnostics of execution. */
  printDiagnostics(): void {
    throw new Error('Not implemented');
  }
}

class CliHtmlOutputExecResult {
  constructor(private inner: CliHtmlOutput | { error: string }) {}
  static fromHtml(html: Document, raw: string): CliHtmlOutputExecResult {
    return new CliHtmlOutputExecResult(new CliHtmlOutput(html, raw));
  }
  /** Gets the result of execution. */
  get result(): CliHtmlOutput | null {
    return this.inner instanceof CliHtmlOutput ? this.inner : null;
  }
  /** Takes the result of execution. */
  takeWarnings(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Takes the error of execution. */
  takeError(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Takes the diagnostics of execution. */
  takeDiagnostics(): NodeError | null {
    throw new Error('Not implemented');
  }
  /** Whether the execution has error. */
  hasError(): boolean {
    return !(this.inner instanceof CliHtmlOutput);
  }
  /** Prints the errors during execution. */
  printErrors(): void {
    'error' in this.inner && console.error(this.inner.error);
  }
  /** Prints the diagnostics of execution. */
  printDiagnostics(): void {
    throw new Error('Not implemented');
  }
}

class CliCompiler {
  private inputs: Record<string, string> = {};
  private fontArgs: Array<string> = [];
  private rootArgs: Array<string> = [];
  needFeature: boolean = true;
  constructor(private args: CompileArgs = {}) {
    this.inputs = { ...this.inputs, ...(args.inputs ?? {}) };
    this.rootArgs = args.workspace ? ['--root', args.workspace] : [];
    // TODO!: add this
    this.fontArgs = [];
  }
  get featureArgs(): Array<string> {
    return this.needFeature ? ['--features', 'html'] : [];
  }
  static create(args?: CompileArgs): CliCompiler {
    return new CliCompiler(args);
  }
  /** Compiles the document as paged target. */
  compile(opts: CompileDocArgs): NodeTypstCompileResult {
    throw new Error('Not implemented');
  }
  /** Compiles the document as html target. */
  compileHtml(opts: CompileDocArgs): CliTypstCompileResult {
    // ??? What's the difference between compileHtml and tryHtml
    return new CliTypstCompileResult({
      error: '`compileHtml` not implemented, use `tryHtml` instead',
    });
  }
  /** Fetches the diagnostics of the document. */
  fetchDiagnostics(opts: NodeError): Array<any> {
    throw new Error('Not implemented');
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
  /** Simply compiles the document as a vector IR. */
  vector(compiledOrBy: NodeTypstDocument | CompileDocArgs): Buffer {
    throw new Error('Not implemented');
  }
  /** Simply compiles the document as a PDF. */
  pdf(compiledOrBy: NodeTypstDocument | CompileDocArgs, opts?: RenderPdfOpts): Buffer {
    throw new Error('Not implemented');
  }
  /** Simply compiles the document as a plain SVG. */
  plainSvg(compiledOrBy: NodeTypstDocument | CompileDocArgs): string {
    throw new Error('Not implemented');
  }
  /** Simply compiles the document as a rich-contented SVG (for browsers). */
  svg(compiledOrBy: NodeTypstDocument | CompileDocArgs): string {
    throw new Error('Not implemented');
  }
  /** Simply compiles the document as a HTML. */
  html(compiledOrBy: NodeTypstDocument | CompileDocArgs): string | null {
    throw new Error('Not implemented');
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
        error: 'Failed to parse the result\n' + `stderr: ${result.stderr.toString()}`,
      });
    }
    return CliHtmlOutputExecResult.fromHtml(parseResult, rawRes);
  }
}

export class CliCompileProvider {
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
  compiler = (): CliCompiler => (this._compiler ||= CliCompiler.create(this.compileArgs));
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
  private _compiler: CliCompiler | undefined = undefined;
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
