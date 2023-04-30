// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_ts_renderer_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_ts_renderer';

import type * as pdfjsModule from 'pdfjs-dist';
import type { InitOptions, BeforeBuildMark } from './options.init';
import { PageViewport } from './viewport';
import { RenderSession } from './internal.types';
import { RenderByStringOptions, RenderOptions, RenderPageOptions } from './options.render';

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
    // console.log(viewport);
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
      // console.log('orignalScale', orignalScale, scale);
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
      doRenderTextLayer(layerList).catch(e => {
        console.error('render text layer', e);
      });

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
