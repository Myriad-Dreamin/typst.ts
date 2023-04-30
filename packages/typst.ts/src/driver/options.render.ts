import { RenderSession } from './internal.types';

/**
 * The options for rendering a page to an image.
 * @typedef {Object} RenderPageOptions
 * @property {number} page_off - The page offset to render.
 */
export class RenderPageOptions {
  page_off: number;
}

/**
 * The options for rendering a Typst document to specified container.
 * @typedef {Object} RenderOptionsBase
 * @property {HTMLDivElement} container - The container to render the Typst document.
 * @property {string} [backgroundColor] - The background color will replace the default one by typst document.
 * @property {number} [pixelPerPt] - The pixel per point scale up the image, which is 2.5 by default and recommended.
 */
export interface RenderOptionsBase {
  container: HTMLDivElement;
  pages?: {
    number: number;
    content: string;
  }[];

  backgroundColor?: string;
  pixelPerPt?: number;
}

/**
 * The options for rendering a Typst document to specified container.
 * @typedef {Object} RenderByStringOptions
 * @property {string} artifactContent - The Typst document content.
 */
export interface RenderByStringOptions extends RenderOptionsBase {
  artifactContent: string;
}

/**
 * The options for rendering a Typst document to specified container.
 * @typedef {Object} RenderInSessionOptions
 * @property {RenderSession} renderSession - The Typst document session that has been created by TypstRenderer.
 */
export interface RenderInSessionOptions extends RenderOptionsBase {
  renderSession: RenderSession;
}

/**
 * The options for rendering a Typst document to specified container.
 * @typedef {Object} RenderByStringOptions
 * @description see {@link RenderByStringOptions} and {@link RenderInSessionOptions}
 */
export type RenderOptions = RenderByStringOptions | RenderInSessionOptions;
