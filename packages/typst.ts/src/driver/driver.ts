// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_ts_renderer_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_ts_renderer';

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
type PageViewportParameters = {
  /**
   * - The xMin, yMin, xMax and
   * yMax coordinates.
   */
  viewBox: Array<number>;
  /**
   * - The scale of the viewport.
   */
  scale: number;
  /**
   * - The rotation, in degrees, of the viewport.
   */
  rotation: number;
  /**
   * - The horizontal, i.e. x-axis, offset. The
   * default value is `0`.
   */
  offsetX?: number | undefined;
  /**
   * - The vertical, i.e. y-axis, offset. The
   * default value is `0`.
   */
  offsetY?: number | undefined;
  /**
   * - If true, the y-axis will not be flipped.
   * The default value is `false`.
   */
  dontFlip?: boolean | undefined;
};
class PageViewport {
  viewBox: number[];
  scale: number;
  rotation: number;
  offsetX: number;
  offsetY: number;
  transform: number[];
  width: number;
  height: number;

  constructor({
    viewBox,
    scale,
    rotation,
    offsetX = 0,
    offsetY = 0,
    dontFlip = false,
  }: PageViewportParameters) {
    this.viewBox = viewBox;
    this.scale = scale;
    this.rotation = rotation;
    this.offsetX = offsetX;
    this.offsetY = offsetY;
    const centerX = (viewBox[2] + viewBox[0]) / 2;
    const centerY = (viewBox[3] + viewBox[1]) / 2;
    let rotateA, rotateB, rotateC, rotateD;
    rotation %= 360;
    if (rotation < 0) {
      rotation += 360;
    }
    switch (rotation) {
      case 180:
        rotateA = -1;
        rotateB = 0;
        rotateC = 0;
        rotateD = 1;
        break;
      case 90:
        rotateA = 0;
        rotateB = 1;
        rotateC = 1;
        rotateD = 0;
        break;
      case 270:
        rotateA = 0;
        rotateB = -1;
        rotateC = -1;
        rotateD = 0;
        break;
      case 0:
        rotateA = 1;
        rotateB = 0;
        rotateC = 0;
        rotateD = -1;
        break;
      default:
        throw new Error('PageViewport: Invalid rotation, must be a multiple of 90 degrees.');
    }
    if (dontFlip) {
      rotateC = -rotateC;
      rotateD = -rotateD;
    }
    let offsetCanvasX, offsetCanvasY;
    let width, height;
    if (rotateA === 0) {
      offsetCanvasX = Math.abs(centerY - viewBox[1]) * scale + offsetX;
      offsetCanvasY = Math.abs(centerX - viewBox[0]) * scale + offsetY;
      width = (viewBox[3] - viewBox[1]) * scale;
      height = (viewBox[2] - viewBox[0]) * scale;
    } else {
      offsetCanvasX = Math.abs(centerX - viewBox[0]) * scale + offsetX;
      offsetCanvasY = Math.abs(centerY - viewBox[1]) * scale + offsetY;
      width = (viewBox[2] - viewBox[0]) * scale;
      height = (viewBox[3] - viewBox[1]) * scale;
    }
    this.transform = [
      rotateA * scale,
      rotateB * scale,
      rotateC * scale,
      rotateD * scale,
      offsetCanvasX - rotateA * scale * centerX - rotateC * scale * centerY,
      offsetCanvasY - rotateB * scale * centerX - rotateD * scale * centerY,
    ];
    this.width = width;
    this.height = height;
  }
  get rawDims() {
    const { viewBox } = this;
    return {
      // todo: shadow
      pageWidth: viewBox[2] - viewBox[0],
      pageHeight: viewBox[3] - viewBox[1],
      pageX: viewBox[0],
      pageY: viewBox[1],
    };
  }
  clone({
    scale = this.scale,
    rotation = this.rotation,
    offsetX = this.offsetX,
    offsetY = this.offsetY,
    dontFlip = false,
  } = {}) {
    return new PageViewport({
      viewBox: this.viewBox.slice(),
      scale,
      rotation,
      offsetX,
      offsetY,
      dontFlip,
    });
  }
  static applyTransform(p: number[], m: number[]) {
    const xt = p[0] * m[0] + p[1] * m[2] + m[4];
    const yt = p[0] * m[1] + p[1] * m[3] + m[5];
    return [xt, yt];
  }
  static applyInverseTransform(p: number[], m: number[]) {
    const d = m[0] * m[3] - m[1] * m[2];
    const xt = (p[0] * m[3] - p[1] * m[2] + m[2] * m[5] - m[4] * m[3]) / d;
    const yt = (-p[0] * m[1] + p[1] * m[0] + m[4] * m[1] - m[5] * m[0]) / d;
    return [xt, yt];
  }
  convertToViewportPoint(x: number, y: number) {
    return PageViewport.applyTransform([x, y], this.transform);
  }
  convertToViewportRectangle(rect: number[]) {
    const topLeft = PageViewport.applyTransform([rect[0], rect[1]], this.transform);
    const bottomRight = PageViewport.applyTransform([rect[2], rect[3]], this.transform);
    return [topLeft[0], topLeft[1], bottomRight[0], bottomRight[1]];
  }
  convertToPdfPoint(x: number, y: number) {
    return PageViewport.applyInverseTransform([x, y], this.transform);
  }
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
      if (!/^#[0-9a-f]{6}$/.test(options.backgroundColor)) {
        throw new Error(
          'Invalid typst.RenderOptions.backgroundColor color for matching ^#[0-9a-f]{6}$ ' +
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
  ): Promise<[RenderResult, any[]]> {
    const pages_info = session.pages_info;
    const page_count = pages_info.page_count;

    /// render each page
    let renderResult = undefined as unknown as RenderResult;

    const doRender = async (i: number, page_off: number) => {
      const canvas = canvasList[i];
      let ctx = canvas.getContext('2d');
      if (ctx) {
        const res = await this.renderImageInSession(session, ctx, {
          page_off,
        });
        if (i === 0) {
          renderResult = {
            width: canvas.width,
            height: canvas.height,
          };
        }
        return res;
      }
    };

    return this.inAnimationFrame(async () => {
      const t = performance.now();

      const textContentList = (
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
              let results = [];
              if (options.pages) {
                for (let i = 0; i < options.pages.length; i++) {
                  results.push(await doRender(i, options.pages[i].number));
                }
              } else {
                for (let i = 0; i < page_count; i++) {
                  results.push(await doRender(i, i));
                }
              }

              return results;
            })(),
          ],
        )
      )[0];

      const t3 = performance.now();

      console.log(`display layer used: render = ${(t3 - t).toFixed(1)}ms`);

      return [renderResult, textContentList];
    });
  }

  private async renderOnePageTextLayer(
    container: HTMLElement,
    // page: pdfjsModule.PDFPageProxy,
    viewport: PageViewport,
    textContentSourceLT: any,
  ) {
    // const textContentSourceGT = await page.getTextContent();
    // console.log(textContentSourceLT, textContentSourceGT, page.getViewport({ scale }));
    console.log(viewport);
    this.pdf.renderTextLayer({
      textContentSource: textContentSourceLT,
      container,

      // todo: implement functions
      // constructor({ viewBox, scale, rotation, offsetX, offsetY, dontFlip, }: PageViewportParameters);
      viewport,
    });
  }

  private async renderTextLayer(
    session: typst.RenderSession,
    container: HTMLDivElement,
    layerList: HTMLDivElement[],
    textSourceList: any[],
    pages?: { number: number }[],
  ) {
    const containerWidth = container.offsetWidth;
    const t2 = performance.now();

    // const buf = await this.renderPdfInSession(session);
    // const doc = await this.pdf.getDocument(buf).promise;
    const t3 = performance.now();

    const pages_info = session.pages_info;

    const renderOne = async (layer: HTMLDivElement, i: number) => {
      // const page = await doc.getPage(i + 1);
      let page_number;
      if (pages) {
        page_number = pages[i].number;
      } else {
        page_number = i;
      }
      const page_info = pages_info.page_by_number(page_number);
      if (!page_info) {
        console.error('page not found for', i, pages_info);
        return;
      }
      const width_pt = page_info.width_pt;
      const height_pt = page_info.height_pt;
      const orignalScale = containerWidth / width_pt;
      // the --scale-factor will truncate our scale, we do it first
      const scale = Number.parseFloat(orignalScale.toFixed(4));
      layer.parentElement?.style.setProperty('--scale-factor', scale.toString());
      console.log('orignalScale', orignalScale, scale);
      const viewport = new PageViewport({
        viewBox: [0, 0, width_pt, height_pt],
        scale: scale,
        offsetX: 0,
        offsetY: 0,
        rotation: 0,
        dontFlip: false,
      });
      this.renderOnePageTextLayer(layer, viewport, textSourceList[i]);
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
    let textContentList: any[];
    const mountContainer = options.container;
    mountContainer.style.visibility = 'hidden';

    const doRenderDisplayLayer = async (
      canvasList: HTMLCanvasElement[],
      resetLayout: () => void,
    ) => {
      try {
        [renderResult, textContentList] = await this.renderDisplayLayer(
          session,
          mountContainer,
          canvasList,
          options,
        );
        resetLayout();
      } finally {
        mountContainer.style.visibility = 'visible';
      }
    };

    const doRenderTextLayer = (layerList: HTMLDivElement[]) =>
      new Promise(resolve => {
        // setTimeout(() => {
        // console.log(textContentList);
        // setImmediate
        this.renderTextLayer(session, mountContainer, layerList, textContentList).then(resolve);
        // }, 0);
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

      /// all children
      const commonDivList = Array.from(container.getElementsByTagName('div')).filter(
        (div: HTMLDivElement) => {
          div.parentElement === container;
        },
      );
      if (!options.pages) {
        container.innerHTML = '';
      }
      container.style.width = '100%';

      // canvas[data-typst-session='{}']

      /// create canvas for each page
      const load_page_cnt = options.pages ? options.pages.length : page_count;
      const canvasList = new Array(load_page_cnt);
      const layerList = new Array(load_page_cnt);
      const commonList = new Array(load_page_cnt);
      const textLayerParentList = new Array(load_page_cnt);

      function createOver(i: number, width: number, height: number, commonDiv: HTMLDivElement) {
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
          const containerWidth = container.offsetWidth;
          const orignalScale = containerWidth / width;
          textLayerParent.style.width = `${containerWidth}px`;
          textLayerParent.style.height = `${height * orignalScale}px`;
          commonDiv.style.width = `${containerWidth}px`;
          commonDiv.style.height = `${height * orignalScale}px`;
          commonDiv.style.position = 'relative';

          // textLayerParent.style.zIndex = '1';
          commonDiv.appendChild(textLayerParent);
          textLayerParent.style.position = 'absolute';
        }
      }

      if (options.pages) {
        for (let i = 0; i < load_page_cnt; i++) {
          const pageNum = options.pages[i].number;
          const pageAst = pages_info.page_by_number(pageNum);
          if (!pageAst) {
            throw new Error(`page ${pageNum} is not loaded`);
          }
          const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
          const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

          // const commonDiv = document.createElement('div');
          let commonDiv = undefined;

          while (pageNum >= commonDivList.length) {
            const elem = document.createElement('div');
            commonDivList.push(elem);
            container.appendChild(elem);
          }
          commonDiv = commonList[i] = commonDivList[pageNum];
          if (commonDiv) {
            commonDiv.innerHTML = '';
          }

          createOver(i, width, height, commonDiv);
        }
      } else {
        for (let i = 0; i < load_page_cnt; i++) {
          const pageAst = pages_info.page(i);
          const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
          const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

          // const commonDiv = document.createElement('div');
          let commonDiv = undefined;

          commonDiv = commonList[i] = document.createElement('div');
          container.appendChild(commonDiv);
          createOver(i, width, height, commonDiv);
        }
      }

      const t2 = performance.now();

      console.log(`layer used: retieve = ${(t2 - t).toFixed(1)}ms`);

      const resetLayout = () => {
        /// resize again to avoid bad width change after render
        if (options.pages) {
          for (let i = 0; i < load_page_cnt; i++) {
            const pageNum = options.pages[i].number;
            const pageAst = pages_info.page_by_number(pageNum);
            if (!pageAst) {
              throw new Error(`page ${pageNum} is not loaded`);
            }
            const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
            const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

            const canvasDiv = canvasList[i].parentElement!;
            const commonDiv = commonList[i];
            const textLayerParent = textLayerParentList[i];

            /// on width change
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
        } else {
          for (let i = 0; i < load_page_cnt; i++) {
            const pageAst = pages_info.page(i);
            const width = Math.ceil(pageAst.width_pt) * imageScaleFactor;
            const height = Math.ceil(pageAst.height_pt) * imageScaleFactor;

            const canvasDiv = canvasList[i].parentElement!;
            const commonDiv = commonList[i];
            const textLayerParent = textLayerParentList[i];

            /// on width change
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
        }
      };

      await doRenderDisplayLayer(canvasList, resetLayout);
      // doRenderTextLayer(layerList).catch(e => {
      //   console.error('render text layer', e);
      // });

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
    const t = performance.now();
    const session = this.renderer.create_session(
      options.artifactContent,
      /* moved */ this.imageOptionsToRust(options),
    );
    if (options.pages) {
      for (const pageInfo of options.pages) {
        this.renderer.load_page(session, pageInfo.number, pageInfo.content);
      }
    }
    const t3 = performance.now();

    console.log(`create session used: render = ${(t3 - t).toFixed(1)}ms`);
    try {
      // console.log(`session`, JSON.stringify(session), `activated`);
      const res = await fn(session);
      // console.log(`session`, JSON.stringify(session), `deactivated`);
      session.free();
      return res;
    } catch (e) {
      // console.log(`session`, JSON.stringify(session), `deactivated by error`, e);
      session.free();
      throw e;
    }
  }
}
