// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_renderer_ts_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_renderer_ts';

import type * as pdfjsModule from 'pdfjs-dist';
import type { InitOptions, BeforeBuildMark } from './options.init';

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

/**
 * The result of rendering a Typst document.
 * @typedef {Object} RenderResult
 * @property {number} width - The width of the rendered Typst document (single page).
 * @property {number} height - The height of the rendered Typst document (single page).
 */
export interface RenderResult {
  width: number;
  height: number;
}

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
 * The interface of Typst renderer.
 * @typedef {Object} TypstRenderer
 * @property {function} init - Initialize the Typst renderer.
 * @property {function} render - Render a Typst document to specified container.
 * @property {function} runWithSession - Run a function with a session to interact
 *   with the wasm module multiple times programmatically.
 */
export interface TypstRenderer {
  init(options?: Partial<InitOptions>): Promise<void>;
  render(options: RenderOptions): Promise<RenderResult>;

  /// run a function with a session, and the sesssion is only available during the
  /// function call.
  ///
  /// the lifetime of session is quite bug-prone, so we current does not make it
  /// longer live than the function call.
  runWithSession<T>(
    options: RenderByStringOptions,
    fn: (session: RenderSession) => Promise<T>,
  ): Promise<T>;
}

/**
 * create a Typst renderer.
 * @param {typeof pdfjsModule} pdf - The pdfjs module.
 * @returns {TypstRenderer} - The Typst renderer.
 * @example
 * ```typescript
 * import { createTypstRenderer } from 'typst';
 * import * as pdfjs from 'pdfjs-dist';
 * const renderer = createTypstRenderer(pdfjs);
 * await renderer.init();
 * await renderer.render({
 *   container: document.getElementById('container'),
 *   artifactContent: '{ ... }',
 * });
 * ```
 */
export function createTypstRenderer(pdf: typeof pdfjsModule): TypstRenderer {
  return new TypstRendererDriver(pdf);
}

const once = <T>(fn: () => T) => {
  let called = false;
  let res: T;
  return () => {
    if (called) {
      return res;
    }
    called = true;
    return (res = fn());
  };
};

const initTypstWasmModule = once(async () => {
  if (typeof typstInit !== 'function') {
    throw new Error('typstInit is not a function');
  }
  await typstInit(typst_wasm_bin);
});

class TypstRendererDriver {
  renderer: typst.TypstRenderer;

  constructor(private pdf: typeof pdfjsModule) {}

  async loadFont(builder: typst.TypstRendererBuilder, fontPath: string): Promise<void> {
    const response = await fetch(fontPath);
    const fontBuffer = new Uint8Array(await response.arrayBuffer());
    await builder.add_raw_font(fontBuffer);
  }

  async init(options?: Partial<InitOptions>): Promise<void> {
    /// init typst wasm module
    await initTypstWasmModule();

    /// build typst renderer
    let builder = new typst.TypstRendererBuilder();
    const buildCtx = { ref: this, builder };

    for (const fn of options?.beforeBuild ?? []) {
      await fn(undefined as unknown as BeforeBuildMark, buildCtx);
    }
    this.renderer = await builder.build();
  }

  private imageOptionsToRust(options: RenderOptions): typst.RenderSessionOptions {
    const rustOptions = new typst.RenderSessionOptions();
    rustOptions.pixel_per_pt = options.pixelPerPt ?? 2;

    if (options.backgroundColor !== undefined) {
      if (!/^#[0-9]{6}$/.test(options.backgroundColor)) {
        throw new Error(
          'Invalid typst.RenderOptions.backgroundColor color for matching ^#[0-9]{6}$ ' +
            options.backgroundColor,
        );
      }

      rustOptions.background_color = options.backgroundColor.slice(1);
    }

    return rustOptions;
  }

  async renderImageInSession(
    session: RenderSession,
    canvas: CanvasRenderingContext2D,
    options?: RenderPageOptions,
  ): Promise<void> {
    if (!options) {
      return this.renderer.render_page_to_canvas(session as typst.RenderSession, canvas);
    }

    const rustOptions = new typst.RenderPageImageOptions();
    rustOptions.page_off = options.page_off;

    return this.renderer.render_page_to_canvas(session as typst.RenderSession, canvas, rustOptions);
  }

  async renderPdf(artifactContent: string): Promise<Uint8Array> {
    return this.renderer.render_to_pdf(artifactContent);
  }

  async renderPdfInSession(session: RenderSession): Promise<Uint8Array> {
    return this.renderer.render_to_pdf_in_session(session as typst.RenderSession);
  }

  private async inAnimationFrame<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      requestAnimationFrame(async () => {
        try {
          resolve(fn());
        } catch (e) {
          reject(e);
        }
      });
    });
  }

  private async renderDisplayLayer(
    session: typst.RenderSession,
    container: HTMLDivElement,
    canvasList: HTMLCanvasElement[],
    options: RenderOptions,
  ): Promise<RenderResult> {
    const pages_info = session.pages_info;
    const page_count = pages_info.page_count;

    return this.inAnimationFrame(async () => {
      const t = performance.now();

      /// render each page
      const renderResult = (
        await Promise.all(
          //   canvasList.map(async (canvas, i) => {
          //     const renderResult = await this.renderImageInSession(session, {
          //       page_off: i,
          //     });
          //     console.log(cyrb53(renderResult.data));
          //     let ctx = canvas.getContext('2d');
          //     if (ctx) {
          //       ctx.putImageData(renderResult, 0, 0);
          //     }

          //     return {
          //       width: renderResult.width,
          //       height: renderResult.height,
          //     };
          //   }),
          // )

          /// seq
          [
            (async () => {
              let i0RenderResult: RenderResult = undefined as unknown as RenderResult;
              for (let i = 0; i < page_count; i++) {
                const canvas = canvasList[i];
                let ctx = canvas.getContext('2d');
                if (ctx) {
                  await this.renderImageInSession(session, ctx, {
                    page_off: i,
                  });
                  if (i === 0) {
                    i0RenderResult = {
                      width: canvas.width,
                      height: canvas.height,
                    };
                  }
                }
              }

              return {
                width: i0RenderResult.width,
                height: i0RenderResult.height,
              };
            })(),
          ],
        )
      )[0]!;

      const t3 = performance.now();

      console.log(`display layer used: render = ${(t3 - t).toFixed(1)}ms`);

      return renderResult;
    });
  }

  private async renderOnePageTextLayer(
    container: HTMLElement,
    page: pdfjsModule.PDFPageProxy,
    scale: number,
  ) {
    const textContentSource = await page.getTextContent();
    console.log({ scale });
    this.pdf.renderTextLayer({
      textContentSource,
      container,
      viewport: page.getViewport({ scale }),
    });
  }

  private async renderTextLayer(
    session: typst.RenderSession,
    container: HTMLDivElement,
    layerList: HTMLDivElement[],
  ) {
    const containerWidth = container.offsetWidth;
    const t2 = performance.now();

    const buf = await this.renderPdfInSession(session);
    const doc = await this.pdf.getDocument(buf).promise;
    const t3 = performance.now();

    const pages_info = session.pages_info;
    const page_count = pages_info.page_count;

    if (page_count === 0) {
      throw new Error(`No page found in session ${session}`);
    }

    const renderOne = async (layer: HTMLDivElement, i: number) => {
      const page = await doc.getPage(i + 1);
      const orignalScale = containerWidth / page.getViewport({ scale: 1 }).width;
      // the --scale-factor will truncate our scale, we do it first
      const scale = Number.parseFloat(orignalScale.toFixed(4));
      layer.parentElement?.style.setProperty('--scale-factor', scale.toString());
      this.renderOnePageTextLayer(layer, page, scale);
    };

    await Promise.all(layerList.map(renderOne));
    const t4 = performance.now();

    console.log(
      `text layer used: retieve/render = ${(t3 - t2).toFixed(1)}/${(t4 - t3).toFixed(1)}ms`,
    );
  }

  async render(options: RenderOptions): Promise<RenderResult> {
    let session: typst.RenderSession;
    let renderResult: RenderResult;
    const mountContainer = options.container;
    mountContainer.style.visibility = 'hidden';

    const doRenderDisplayLayer = async (canvasList: HTMLCanvasElement[]) => {
      renderResult = await this.renderDisplayLayer(session, mountContainer, canvasList, options);
    };

    const doRenderTextLayer = (layerList: HTMLDivElement[]) =>
      new Promise(resolve => {
        setTimeout(() => {
          // setImmediate
          this.renderTextLayer(session, mountContainer, layerList).then(resolve);
        }, 0);
      });

    return this.withinOptionSession(options, async sessionRef => {
      session = sessionRef;

      const container = mountContainer;

      const imageScaleFactor = options.pixelPerPt ?? 2;

      const pages_info = session.pages_info;
      const page_count = pages_info.page_count;

      if (page_count === 0) {
        throw new Error(`No page found in session ${session}`);
      }

      const t = performance.now();

      container.innerHTML = '';
      container.style.width = '100%';

      /// create canvas for each page
      const canvasList = new Array(page_count);
      const layerList = new Array(page_count);
      const commonList = new Array(page_count);
      const textLayerParentList = new Array(page_count);
      for (let i = 0; i < page_count; i++) {
        const pageAst = pages_info.page(i);
        const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
        const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

        // const commonDiv = document.createElement('div');
        const commonDiv = (commonList[i] = document.createElement('div'));
        const canvas = (canvasList[i] = document.createElement('canvas'));
        const textLayer = (layerList[i] = document.createElement('div'));
        const textLayerParent = (textLayerParentList[i] = document.createElement('div'));

        const ctx = canvas.getContext('2d');
        if (ctx) {
          const canvasDiv = document.createElement('div');

          canvas.width = width;
          canvas.height = height;

          canvasDiv.appendChild(canvas);
          // canvasDiv.style.zIndex = '-1';
          // canvas.style.zIndex = '-1';
          commonDiv.appendChild(canvasDiv);
          canvasDiv.style.position = 'absolute';
        }

        {
          textLayerParent.appendChild(textLayer);

          textLayerParent.className = 'text-layer textLayer';

          /// on width change
          console.log('resize', i, container.offsetWidth);
          const containerWidth = container.offsetWidth;
          const orignalScale = containerWidth / width;
          textLayerParent.style.width = `${containerWidth}px`;
          textLayerParent.style.height = `${height * orignalScale}px`;
          commonDiv.style.width = `${containerWidth}px`;
          commonDiv.style.height = `${height * orignalScale}px`;

          // textLayerParent.style.zIndex = '1';
          commonDiv.appendChild(textLayerParent);
          textLayerParent.style.position = 'absolute';
        }

        commonDiv.style.position = 'relative';
        container.appendChild(commonDiv);
        // const clearDiv = document.createElement('div');
        // clearDiv.style.zIndex = '-1';
        // container.appendChild(clearDiv);
      }

      const t2 = performance.now();

      console.log(`layer used: retieve = ${(t2 - t).toFixed(1)}ms`);

      await Promise.all([doRenderDisplayLayer(canvasList), doRenderTextLayer(layerList)]);

      /// resize again to avoid bad width change after render
      for (let i = 0; i < page_count; i++) {
        const pageAst = pages_info.page(i);
        const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
        const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

        const canvasDiv = canvasList[i].parentElement!;
        const commonDiv = commonList[i];
        const textLayerParent = textLayerParentList[i];

        /// on width change
        console.log('resize', i, container.offsetWidth);
        const containerWidth = container.offsetWidth;
        const orignalScale = containerWidth / width;
        textLayerParent.style.width = `${containerWidth}px`;
        textLayerParent.style.height = `${height * orignalScale}px`;
        commonDiv.style.width = `${containerWidth}px`;
        commonDiv.style.height = `${height * orignalScale}px`;

        // compute scaling factor according to the paper size
        const currentScale = container.offsetWidth / width;
        canvasDiv.style.transformOrigin = '0px 0px';
        canvasDiv.style.transform = `scale(${currentScale})`;
      }

      mountContainer.style.visibility = 'visible';
      return renderResult;
    });
  }

  private withinOptionSession<T>(
    options: RenderOptions,
    fn: (session: typst.RenderSession) => Promise<T>,
  ): Promise<T> {
    if ('renderSession' in options) {
      return fn(options.renderSession as typst.RenderSession);
    }

    if ('artifactContent' in options) {
      return this.runWithSession(options, fn);
    }

    throw new Error(
      'Invalid render options, should be one of RenderByStringOptions|RenderBySessionOptions',
    );
  }

  async runWithSession<T>(
    options: RenderByStringOptions,
    fn: (session: typst.RenderSession) => Promise<T>,
  ): Promise<T> {
    const session = this.renderer.create_session(
      options.artifactContent,
      /* moved */ this.imageOptionsToRust(options),
    );
    try {
      console.log(`session`, JSON.stringify(session), `activated`);
      const res = await fn(session);
      console.log(`session`, JSON.stringify(session), `deactivated`);
      session.free();
      return res;
    } catch (e) {
      console.log(`session`, JSON.stringify(session), `deactivated by error`, e);
      session.free();
      throw e;
    }
  }
}
