import { RenderSession } from './internal.types';

const vectorFormats = ['vector'] as const;
export type VectorFormat = (typeof vectorFormats)[0];

/**
 * The options for creating a session.
 * @property {string} [format] - specify the format of render data
 *   + `vector`: decode {@link CreateSessionOptions.artifactContent} in binary vector format
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

  backgroundColor?: string;
  pixelPerPt?: number;
}

/**
 * The options for rendering a preprocessed Typst document to specified container.
 * @property {HTMLElement} [container] - The container to render the Typst document.
 */
export interface RenderToSvgOptions {
  container: HTMLElement;
}

/**
 * The options for rendering a page to an image.
 * @property {number} page_off - The page offset to render.
 */
export class RenderPageOptions {
  page_off: number;
}
