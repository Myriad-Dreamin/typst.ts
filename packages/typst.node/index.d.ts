/* auto-generated by NAPI-RS */
/* eslint-disable */

/**
 * A nullable boxed compiler wrapping.
 *
 * This is for transferring boxed compiler between functions.
 * It will panic if the inner boxed compiler is already taken.
 */
export class BoxedCompiler {}
export type JsBoxedCompiler = BoxedCompiler;

export class DynLayoutCompiler {
  /** Creates a new compiler based on the given arguments. */
  static fromBoxed(b: BoxedCompiler): DynLayoutCompiler;
  /** Exports the document as a vector IR containing multiple layouts. */
  vector(compileBy: CompileDocArgs): Buffer;
}

/** Node wrapper to access compiler interfaces. */
export class NodeCompiler {
  /**
   * Creates a new compiler based on the given arguments.
   *
   * == Example
   *
   * Creates a new compiler with default arguments:
   * ```ts
   * const compiler = NodeCompiler.create();
   * ```
   *
   * Creates a new compiler with custom arguments:
   * ```ts
   * const compiler = NodeCompiler.create({
   *   workspace: '/path/to/workspace',
   * });
   * ```
   */
  static create(args?: CompileArgs | undefined | null): NodeCompiler;
  /** Casts the inner compiler. */
  static fromBoxed(b: BoxedCompiler): NodeCompiler;
  /** Takes ownership of the inner compiler. */
  intoBoxed(): BoxedCompiler;
  /**
   * Evict the **global** cache.
   *
   * This removes all memoized results from the cache whose age is larger
   * than or equal to `max_age`. The age of a result grows by one during
   * each eviction and is reset to zero when the result produces a cache
   * hit. Set `max_age` to zero to completely clear the cache.
   *
   * A suggested `max_age` value for regular non-watch tools is `10`.
   * A suggested `max_age` value for regular watch tools is `30`.
   */
  evictCache(maxAge: number): void;
  /**
   * Adds a source file to the compiler.
   * @param path - The path of the source file.
   * @param source - The source code of the source file.
   */
  addSource(path: string, source: string): void;
  /**
   * Adds a shadow file to the compiler.
   * @param path - The path to the shadow file.
   * @param content - The content of the shadow file.
   */
  mapShadow(path: string, content: Buffer): void;
  /**
   * Removes a shadow file from the compiler.
   * @param path - The path to the shadow file.
   */
  unmapShadow(path: string): void;
  /**
   * Resets the shadow files.
   * Note: this function is independent to the {@link reset} function.
   */
  resetShadow(): void;
  /** Compiles the document. */
  compile(opts: CompileDocArgs): NodeTypstCompileResult;
  /** Fetches the diagnostics of the document. */
  fetchDiagnostics(opts: NodeError): Array<any>;
  /** Queries the data of the document. */
  query(compiledOrBy: NodeTypstDocument | CompileDocArgs, args: QueryDocArgs): any;
  /** Simply compiles the document as a vector IR. */
  vector(compiledOrBy: NodeTypstDocument | CompileDocArgs): Buffer;
  /** Simply compiles the document as a PDF. */
  pdf(compiledOrBy: NodeTypstDocument | CompileDocArgs, opts?: RenderPdfOpts): Buffer;
  /** Simply compiles the document as a plain SVG. */
  plainSvg(compiledOrBy: NodeTypstDocument | CompileDocArgs): string;
  /** Simply compiles the document as a rich-contented SVG (for browsers). */
  svg(compiledOrBy: NodeTypstDocument | CompileDocArgs): string;
}

/** A node error. */
export class NodeError {
  /** Gets the kind of the error. */
  get kind(): string;
  /**
   * Gets the short diagnostics of the error.
   *
   * To retrieve the full diagnostics, please use
   * `NodeCompiler.fetch_diagnostics`.
   */
  get shortDiagnostics(): Array<any>;
  /**
   * Gets the compilation status
   *
   * If the error is an error, it will return `internal_error`.
   *
   * Otherwise, if diagnostics contains any error, it will return `error`.
   *
   * Otherwise, if diagnostics contains any warning, it will return
   * `warning`.
   *
   * Otherwise, it will return `ok`.
   */
  get compilationStatus(): string;
}

/** Result of single typst compilation. */
export class NodeTypstCompileResult {
  /** Gets the result of compilation. */
  get result(): NodeTypstDocument | null;
  /** Takes the diagnostics of compilation. */
  takeDiagnostics(): NodeError | null;
}

/** A shared typst document object. */
export class NodeTypstDocument {
  /** Gets the number of pages in the document. */
  get numOfPages(): number;
  /** Gets the title of the document. */
  get title(): string | null;
  /** Gets the authors of the document. */
  get authors(): Array<string> | null;
  /** Gets the keywords of the document. */
  get keywords(): Array<string> | null;
  /**
   * Gets the unix timestamp (in nanoseconds) of the document.
   *
   * Note: currently typst doesn't specify the timezone of the date, and we
   * keep stupid and doesn't add timezone info to the date.
   */
  get date(): number | null;
  /**
   * Determines whether the date should be automatically generated.
   *
   * This happens when user specifies `date: auto` in the document
   * explicitly.
   */
  get enabledAutoDate(): boolean;
}

export interface CompileArgs {
  /** Adds additional directories to search for fonts */
  fontArgs?: Array<NodeAddFontPaths | NodeAddFontBlobs>;
  /** Path to typst workspace. */
  workspace?: string;
  /** Adds a string key-value pair visible through `sys.inputs` */
  inputs?: Record<string, string>;
}

/**
 * Arguments to compile a document.
 *
 * If no `mainFileContent` or `mainFilePath` is specified, the compiler will
 * use the entry file specified in the constructor of `NodeCompiler`.
 */
export interface CompileDocArgs {
  /**
   * Directly specify the main file content.
   * Exclusive with `mainFilePath`.
   */
  mainFileContent?: string;
  /**
   * Path to the entry file.
   * Exclusive with `mainFileContent`.
   */
  mainFilePath?: string;
  /** Add a string key-value pair visible through `sys.inputs`. */
  inputs?: Record<string, string>;
}

export interface NodeAddFontBlobs {
  /** Adds additional memory fonts */
  fontBlobs: Array<Array<number>>;
}

export interface NodeAddFontPaths {
  /** Adds additional directories to search for fonts */
  fontPaths: Array<string>;
}

/** Arguments to query the document. */
export interface QueryDocArgs {
  /** The query selector. */
  selector: string;
  /** An optional field to select on the element of the resultants. */
  field?: string;
}

/** Arguments to render a PDF. */
export interface RenderPdfOpts {
  /**
   * An optional (creation) timestamp to be used in the PDF.
   *
   * This is used when you *enable auto timestamp* in the document.
   */
  creationTimestamp?: number;
}
