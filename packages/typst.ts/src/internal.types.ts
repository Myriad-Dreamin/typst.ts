export const kObject = Symbol.for('reflexo-obj');

/**
 * The session kernel of a Typst document.
 * @typedef {Object} RenderSessionKernel
 * @property {string} background_color - The background color of the Typst
 * document.
 * @property {number} pixel_per_pt - The pixel per point scale up the image.
 *
 * caution: the underlying object is created by the wasm module, which means
 * that:
 *   + any modification will raise an error.
 *   + Never clone the object and pass it back to typst renderer.
 *   + You must not hold a reference, since it will be freed after a while
 */
export interface RenderSessionKernel {
  /**
   * Set the background color of the Typst document.
   * @param {string} - The background color in format of `^#?[0-9a-f]{6}$`
   *
   * Note: Default to `#ffffff`.
   *
   * Note: Only available in canvas rendering mode.
   */
  readonly background_color: string;

  /**
   * Set the pixel per point scale up the canvas panel.
   *
   * Note: Default to `3`.
   *
   * Note: Only available in canvas rendering mode.
   */
  readonly pixel_per_pt: number;
}

/**
 * The page information of a Typst document.
 * @typedef {Object} PageInfo
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

export interface Point {
  x: number;
  y: number;
}

export interface Rect {
  lo: Point;
  hi: Point;
}
