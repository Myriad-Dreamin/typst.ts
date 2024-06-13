export const kObject = Symbol.for('reflexo-obj');

/**
 * The page information of a Typst document.
 * @property {number} pageOffset - The offset of the page.
 * @property {number} width - The width of the page in pt.
 * @property {number} height - The height of the page in pt.
 */
export class PageInfo {
  pageOffset: number;
  width: number;
  height: number;
}

export interface FsAccessModel {
  getMTime(path: string): Date | undefined;
  isFile(path: string): boolean | undefined;
  getRealPath(path: string): string | undefined;
  readAll(path: string): Uint8Array | undefined;
}

export interface PackageSpec {
  namespace: string;
  name: string;
  version: string;
}

export interface PackageResolveContext {
  untar(data: Uint8Array, cb: (path: string, data: Uint8Array, mtime: number) => void): void;
}

export interface PackageRegistry {
  resolve(path: PackageSpec, context: PackageResolveContext): string | undefined;
}

export interface Point {
  x: number;
  y: number;
}

export interface Rect {
  lo: Point;
  hi: Point;
}

export type TransformMatrix = [number, number, number, number, number, number];

//#region Semantic tokens: https://github.com/microsoft/vscode/issues/86415
export interface SemanticTokensLegend {
  readonly tokenTypes: string[];
  readonly tokenModifiers: string[];
}

export interface SemanticTokens {
  /**
   * The result id of the tokens.
   *
   * This is the id that will be passed to `DocumentSemanticTokensProvider.provideDocumentSemanticTokensEdits` (if implemented).
   */
  readonly resultId?: string;
  readonly data: Uint32Array;
}
//#endregion

/**
 * The result of rendering a Typst document to a canvas.
 */
export interface RenderCanvasResult {
  cacheKey: string;
  htmlSemantics: string[];
}

export const enum TypstDefaultParams {
  PIXEL_PER_PT = 3,
}
