import type { QueryDocArgs } from '@myriaddreamin/typst-ts-node-compiler';
import { CompileArgs } from '@myriaddreamin/typst-ts-node-compiler';
import * as path from 'path';
import { ResolvedTypstInput } from './input.js';

/**
 * The callback to be called when the document is compiled.
 *
 * @param mainFilePath The source file path
 * @param project The compiling project (document)
 * @param ctx The compile provider
 */
export type OnCompileCallback<P extends CompileProvider<P>, T = void> = (
  mainFilePath: ResolvedTypstInput,
  project: TypstHTMLCompiler,
  ctx: P,
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

export interface HtmlOutputExecResult {
  /** Gets the result of execution. */
  get result(): HtmlOutput | null;
  /** Whether the execution has error. */
  hasError(): boolean;
  /** Prints the diagnostics of execution. */
  printDiagnostics(): void;
}

export interface TypstHTMLCompiler {
  // Add svg here?
  query(doc: ResolvedTypstInput, args: QueryDocArgs): any;
  tryHtml(doc: ResolvedTypstInput): HtmlOutputExecResult;
}

export interface TypstHTMLWatcher {}

export abstract class CompileProvider<P extends CompileProvider<P>> {
  compiled = new Map<string, string>();

  constructor(
    public readonly onCompile: OnCompileCallback<P>,
    readonly compileArgs: CompileArgs,
    public inputRoot: string = '.',
  ) {}

  resolveRel(input: string, ext = '.html') {
    const rel = input.endsWith('.typ') ? input.slice(0, -4) : input;
    return path.relative(this.inputRoot, rel + ext);
  }

  /**
   * Lazily created compiler.
   */
  abstract compiler(): TypstHTMLCompiler;
  /**
   * Lazily created watcher
   */
  abstract watcher(): TypstHTMLWatcher;

  abstract isWatch: boolean;

  /**
   * Compiles the source file to the destination file.
   *
   * @param {ResolvedTypstInput} input The resolved input
   *
   * @example
   * compile("src/index.typ", "dist/index.html")(compiler());
   */
  abstract compile(input: ResolvedTypstInput): (compiler: TypstHTMLCompiler) => void;

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
