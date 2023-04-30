/**
 * The session of a Typst document.
 * @typedef {Object} RenderSession
 * @property {string} background_color - The background color of the Typst document.
 * @property {number} pixel_per_pt - The pixel per point scale up the image.
 *
 * caution: the underlying object is created by the wasm module, which means that
 *   + any modification will raise an error.
 *   + Never clone the object and pass it back to typst renderer.
 *   + You must not hold a reference, since it will be freed after a while
 */
export interface RenderSession {
  readonly background_color: string;
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
