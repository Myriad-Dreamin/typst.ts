import { RenderSession } from './internal.types';

/**
 * The options for rendering a page to an image.
 * @property {number} page_off - The page offset to render.
 */
export class RenderPageOptions {
  page_off: number;
}

const artifactFormats = ['serde_json', 'js', 'ir'] as const;
export type ArtifactFormat = (typeof artifactFormats)[0 | 1 | 2];

export function isRenderArtifactOption(f: RenderOptionsBase): f is RenderArtifactOptionsBase {
  return artifactFormats.includes(f.format as any);
}

const svgFormats = ['vector'] as const;
export type SvgFormat = (typeof svgFormats)[0];

export function isRenderSvgOption(f: RenderOptionsBase): f is RenderSvgOptionsBase {
  return svgFormats.includes(f.format as any);
}

// createModule(b?: Uint8Array): Promise<unknown> {
//   return new Promise(resolve => {
//     resolve(b ? this.renderer.create_svg_session(b) : this.renderer.create_empty_svg_session());
//   });
// }

/**
 * The options for rendering a Typst document to specified container.
 * see {@link RenderArtifactOptionsBase} and {@link RenderSvgOptionsBase}
 */
export type RenderOptionsBase = RenderArtifactOptionsBase | RenderSvgOptionsBase;

/**
 * The options for rendering a preprocessed Typst document to specified container.
 * @property {HTMLDivElement} [container] - The container to render the Typst document.
 * @property {string} [backgroundColor] - The background color will replace the default one by typst document.
 * @property {number} [pixelPerPt] - The pixel per point scale up the image, which is 2.5 by default and recommended.
 * @property {string} [format] - specify the format of render data
 *   + `serde_json`: decode {@link RenderByContentOptions.artifactContent} via `serde_json`
 *   + `js`: decode {@link RenderByContentOptions.artifactContent} via `JSON.parse`
 *   + `ir`: decode {@link RenderByContentOptions.artifactContent} in artifact_ir format
 */
export interface RenderArtifactOptionsBase {
  container: HTMLDivElement;

  backgroundColor?: string;
  pixelPerPt?: number;

  format: ArtifactFormat;
}

/**
 * The options for rendering a preprocessed Typst document to specified container.
 * @property {HTMLDivElement} [container] - The container to render the Typst document.
 * @property {string} [format] - specify the format of render data
 *   + `svg_text`: decode {@link RenderByContentOptions.artifactContent} in text svg format
 *   + `vector`: decode {@link RenderByContentOptions.artifactContent} in binary vector format
 */
export interface RenderSvgOptionsBase {
  container: HTMLDivElement;

  format: SvgFormat;
}

interface RenderByArtifactContentOptions extends RenderArtifactOptionsBase {
  artifactContent: Uint8Array;
}

interface RenderBySvgContentOptions extends RenderSvgOptionsBase {
  artifactContent?: Uint8Array;
}

/**
 * The options for rendering a Typst document to specified container.
 * @property {Uint8Array} artifactContent - The Typst document content.
 */
export type RenderByContentOptions<T = ArtifactFormat | SvgFormat> =
  | (RenderByArtifactContentOptions & { format: T })
  | (RenderBySvgContentOptions & { format: T });

/**
 * The options for rendering a Typst document to specified container.
 * @property {RenderSession} renderSession - The Typst document session that has been created by TypstRenderer.
 */
export type RenderInSessionOptions<T> = RenderOptionsBase & {
  format: T;
  renderSession: RenderSession;
};

/**
 * The options for rendering a Typst document to specified container.
 * @description see {@link RenderByContentOptions} and {@link RenderInSessionOptions}
 */
export type RenderOptions<T = ArtifactFormat | SvgFormat> =
  | RenderByContentOptions<T>
  | RenderInSessionOptions<T>;
