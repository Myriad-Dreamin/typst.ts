import { Rect } from './internal.types.mjs';
import { RenderSession } from './renderer.mjs';

const vectorFormats = ['vector'] as const;
export type VectorFormat = (typeof vectorFormats)[0];

/**
 * The options for creating a session.
 * @property {string} [format] - specify the format of render data
 *   + `vector`: decode {@link CreateSessionOptions['artifactContent']} in binary vector format
 * @property {Uint8Array} artifactContent - The artifact content of Typst document.
 */
export interface CreateSessionOptions<T = VectorFormat> {
  format: T;
  artifactContent: Uint8Array;
}

/**
 * The options for rendering a Typst.
 * @description see {@link RenderByContentOptions} and {@link RenderInSessionOptions}
 */
export type RenderOptions<Base = RenderToCanvasOptions> =
  | RenderInSessionOptions<Base>
  | RenderByContentOptions<Base>;

/**
 * The options for rendering a Typst document with a created session.
 * @property {RenderSession} renderSession - The Typst document session that has been created by TypstRenderer.
 */
export type RenderInSessionOptions<Base = RenderToCanvasOptions> = Base & {
  renderSession: RenderSession;
};

/**
 * The options for rendering a Typst document by artifact content.
 * See {@link CreateSessionOptions} for more details.
 */
export type RenderByContentOptions<Base = RenderToCanvasOptions> = Base & CreateSessionOptions;

/**
 * The options for rendering a preprocessed Typst document to specified container.
 * @property {HTMLElement} [container] - The container to render the Typst document.
 * @property {string} [backgroundColor] - The background color will replace the default one by typst document.
 * @property {number} [pixelPerPt] - The pixel per point scale up the image, which is 2.5 by default and recommended.
 */
export interface RenderToCanvasOptions {
  container: HTMLElement;

  /**
   * Set the background color in format of `^#?[0-9a-f]{6}$`
   *
   * Note: Default to `#ffffff`.
   */
  backgroundColor?: string;

  /**
   * Set the pixel per point scale up the canvas panel.
   *
   * Note: Default to `3`.
   */
  pixelPerPt?: number;
}

/**
 * The options for rendering a svg string.
 */
export interface RenderSvgOptions {
  window?: Rect;
}

/**
 * The options for rendering a preprocessed Typst document to specified container.
 * @property {HTMLElement} [container] - The container to render the Typst document.
 */
export interface RenderToSvgOptions extends RenderSvgOptions {
  container: HTMLElement;
}

/**
 * The options for manipulating the Typst document in the session.
 */
export interface ManipulateDataOptions {
  /**
   * The action to manipulate the data.
   * @description `reset`: reset the data to the initial state.
   * @description `merge`: merge the data to the current state.
   * @default 'reset'
   */
  action?: 'reset' | 'merge';
  /**
   * Opaque data to manipulate the Typst document from server.
   */
  data: Uint8Array;
}

/**
 * The options for rendering a page to an image.
 * @property {number} page_off - The page offset to render.
 */
export class RenderPageOptions {
  page_off: number;
}
