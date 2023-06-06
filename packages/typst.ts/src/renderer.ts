// @ts-ignore
import typstInit, * as typst from '../../renderer/pkg/typst_ts_renderer';

import type * as pdfjsModule from 'pdfjs-dist';
import type { InitOptions } from './options.init';
import { PageViewport } from './viewport';
import { PageInfo, RenderSession } from './internal.types';
import {
  RenderByContentOptions,
  RenderOptions,
  RenderOptionsBase,
  RenderPageOptions,
} from './options.render';
import { RenderView, renderTextLayer } from './view';
import { LazyWasmModule } from './wasm';
import { buildComponent } from './init';

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
  loadGlyphPack(pack: unknown): Promise<void>;

  render(options: RenderOptions): Promise<RenderResult>;

  /// run a function with a session, and the sesssion is only available during the
  /// function call.
  ///
  /// the lifetime of session is quite bug-prone, so we current does not make it
  /// longer live than the function call.
  runWithSession<T>(
    options: RenderByContentOptions,
    fn: (session: RenderSession) => Promise<T>,
  ): Promise<T>;
}

const gRendererModule = new LazyWasmModule(typstInit);

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
export function createTypstRenderer(pdf: unknown): TypstRenderer {
  return new TypstRendererDriver(pdf as typeof pdfjsModule);
}

class TypstRendererDriver {
  renderer: typst.TypstRenderer;

  constructor(private pdf: typeof pdfjsModule) {}

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.renderer = await buildComponent(options, gRendererModule, typst.TypstRendererBuilder, {});
  }

  loadGlyphPack(pack: unknown): Promise<void> {
    this.renderer.load_glyph_pack(pack);
    return Promise.resolve();
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

    if (options.format !== undefined) {
      rustOptions.format = options.format;
    }

    return rustOptions;
  }

  loadPagesInfo(session: typst.RenderSession, options: RenderOptionsBase): PageInfo[] {
    const pages_info = session.pages_info;
    const pageInfos: PageInfo[] = [];
    if (options.pages) {
      for (let i = 0; i < options.pages.length; i++) {
        const pageNum: number = options.pages[i].number;
        const pageAst = pages_info.page_by_number(pageNum);
        if (!pageAst) {
          throw new Error(`page ${pageNum} is not loaded`);
        }
        pageInfos.push({
          pageOffset: pageAst.page_off,
          width: pageAst.width_pt,
          height: pageAst.height_pt,
        });
      }
    } else {
      const pageCount = pages_info.page_count;
      for (let i = 0; i < pageCount; i++) {
        const pageAst = pages_info.page(i);
        pageInfos.push({
          pageOffset: pageAst.page_off,
          width: pageAst.width_pt,
          height: pageAst.height_pt,
        });
      }
    }

    return pageInfos;
  }

  renderImageInSession(
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

  // async renderPdf(artifactContent: string): Promise<Uint8Array> {
  // return this.renderer.render_to_pdf(artifactContent);
  // }
  //
  // async renderPdfInSession(session: RenderSession): Promise<Uint8Array> {
  // return this.renderer.render_to_pdf_in_session(session as typst.RenderSession);
  // }

  private async inAnimationFrame<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      requestAnimationFrame(() => {
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
      const ctx = canvas.getContext('2d');
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
              const results = [];
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

  private renderOnePageTextLayer(
    container: HTMLElement,
    viewport: PageViewport,
    textContentSource: any,
  ) {
    // console.log(viewport);
    this.pdf.renderTextLayer({
      textContentSource,
      container,
      viewport,
    });
  }

  private renderTextLayer(
    session: typst.RenderSession,
    view: RenderView,
    container: HTMLDivElement,
    layerList: HTMLDivElement[],
    textSourceList: any[],
  ) {
    renderTextLayer(this.pdf, container, view.pageInfos, layerList, textSourceList);
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

    return this.withinOptionSession(options, async sessionRef => {
      session = sessionRef;
      if (session.pages_info.page_count === 0) {
        throw new Error(`No page found in session`);
      }

      const t = performance.now();

      const pageView = new RenderView(
        this.loadPagesInfo(session, options),
        mountContainer,
        options,
      );
      const t2 = performance.now();

      console.log(`layer used: retieve = ${(t2 - t).toFixed(1)}ms`);

      await doRenderDisplayLayer(pageView.canvasList, () => pageView.resetLayout());
      this.renderTextLayer(session, pageView, mountContainer, pageView.layerList, textContentList);

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
      'Invalid render options, should be one of RenderByContentOptions|RenderBySessionOptions',
    );
  }

  async runWithSession<T>(
    options: RenderByContentOptions,
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
