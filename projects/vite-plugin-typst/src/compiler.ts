import type { CompileArgs, QueryDocArgs } from '@myriaddreamin/typst-ts-node-compiler';
import * as path from 'path';
import { ResolvedTypstInput } from './input.js';
import { ExecResult } from './index.js';

/**
 * The callback to be called when the document is compiled.
 *
 * @param mainFilePath The source file path
 * @param project The compiling project (document)
 * @param ctx The compile provider
 */
export type OnCompileCallback<T = void, Ctx extends CompileProvider = CompileProvider> = (
  mainFilePath: ResolvedTypstInput,
  project: Ctx['ProjectType'],
  ctx: Ctx,
) => T;

export type PartialCallback<T = void> = (
  mainFilePath: ResolvedTypstInput,
  project: TypstHTMLCompiler<any>,
) => T;

export interface HtmlOutput {
  /** Gets the title of the document. */
  title(): string | null;
  /** Gets the description of the document. */
  description(): string | null;
  /** Gets the body of the document. */
  body(): string;
  /** Gets the body of the document as bytes. */
  bodyBytes(): Buffer;
  /** Gets the HTML of the document. */
  html(): string;
  /** Gets the HTML of the document as bytes. */
  htmlBytes(): Buffer;
}

export type HtmlOutputExecResult = ExecResult<HtmlOutput | null>;

export interface TypstHTMLCompiler<Doc = any> {
  // Add svg here?
  query(doc: Doc | ResolvedTypstInput, args: QueryDocArgs): any;
  tryHtml(doc: ResolvedTypstInput): ExecResult<HtmlOutput>;
}

export interface TypstHTMLWatcher {
  add(paths: string[], exec: (project: TypstHTMLCompiler<any>) => void): void;
  watch(): void;
  clear(): void;
}

export type CompileProviderConstructor<Ctx extends CompileProvider = CompileProvider> = new (
  isWatch: boolean,
  compileArgs: CompileArgs,
  onCompile: OnCompileCallback<void, Ctx>,
  inputRoot?: string,
) => Ctx;

/**
 * The common interface for the compile provider.
 */
export abstract class CompileProvider {
  OutputType: HtmlOutput = undefined!;
  ProjectType: TypstHTMLCompiler<unknown> = undefined!;

  static kind = Symbol.for('vite-plugin-typst:CompileProvider');

  compiled = new Map<string, string>();

  constructor(
    public isWatch: boolean,
    readonly compileArgs: CompileArgs,
    public readonly onCompile: OnCompileCallback,
    public inputRoot: string = '.',
  ) {}

  static isCons<Ctx extends CompileProvider>(cls: any): cls is CompileProviderConstructor<Ctx> {
    return cls?.kind === CompileProvider.kind;
  }

  resolveRel(input: string, ext = '.html') {
    const rel = input.endsWith('.typ') ? input.slice(0, -4) : input;
    return path.relative(this.inputRoot, rel + ext);
  }

  /**
   * Lazily created compiler.
   */
  abstract compiler(): TypstHTMLCompiler<any>;
  /**
   * Lazily created watcher
   */
  abstract watcher(): TypstHTMLWatcher;

  /**
   * Compiles the source file to the destination file.
   *
   * @param {ResolvedTypstInput} input The resolved input
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  abstract compile(input: ResolvedTypstInput): (compiler: TypstHTMLCompiler<any>) => void;

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
  abstract compileOrWatch(input: ResolvedTypstInput): void;
}
