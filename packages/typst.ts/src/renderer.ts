// @ts-ignore
import typstInit, * as typst from '@myriaddreamin/typst-ts-renderer';

import type { InitOptions } from './options.init';
import { PageViewport } from './viewport';
import { PageInfo, RenderSession } from './internal.types';
import {
  CreateSessionOptions,
  RenderToCanvasOptions,
  RenderOptions,
  RenderPageOptions,
  RenderToSvgOptions,
  ManipulateDataOptions,
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

export interface RenderPageResult {
  textContent: any;
  annotationList: AnnotationList;
}

/**
 * The interface of Typst renderer.
 */
export interface TypstRenderer extends TypstSvgRenderer {
  /**
   * Initialize the Typst renderer.
   * @param {Partial<InitOptions>} options - The options for initializing the
   * Typst renderer.
   */
  init(options?: Partial<InitOptions>): Promise<void>;

  /**
   * Load a glyph pack for all of the Typst documents to render.
   * Note: this function is still under development.
   * @param pack
   */
  loadGlyphPack(pack: unknown): Promise<void>;

  /**
   * Render a Typst document to canvas.
   * @param {RenderOptions<RenderToCanvasOptions>} options - The options for
   * rendering a Typst document to specified container.
   * @returns {RenderResult} - The result of rendering a Typst document.
   * @example
   * ```typescript
   * let fetchDoc = (path) => fetch(path).then(
   *   response => new Uint8Array(response.arrayBuffer()))
   * renderer.renderToCanvas({
   *   container: document.getElementById('container'),
   *   pixelPerPt: 3,
   *   backgroundColor: '#ffffff',
   *   artifactContent: await fetchDoc('typst-main.sir.in'),
   * });
   * ```
   */
  renderToCanvas(options: RenderOptions<RenderToCanvasOptions>): Promise<void>;

  /**
   * Render a Typst document to svg.
   * @param {RenderOptions<RenderToSvgOptions>} options - The options for
   * rendering a Typst document to specified container.
   * @returns {RenderResult} - The result of rendering a Typst document.
   * @example
   * ```typescript
   * let fetchDoc = (path) => fetch(path).then(
   *   response => new Uint8Array(response.arrayBuffer()))
   * renderer.renderToSvg({
   *   container: document.getElementById('container'),
   *   artifactContent: await fetchDoc('typst-main.sir.in'),
   * });
   * ```
   */
  renderToSvg(options: RenderOptions<RenderToSvgOptions>): Promise<void>;

  /**
   * Manipulate the Typst document in the session.
   * See {@link ManipulateDataOptions} for more details.
   * @param {RenderSession} session - The Typst document session that has been
   * created by TypstRenderer.
   * @param {ManipulateDataOptions} opts - The options for manipulating the
   * Typst document in the session.
   *
   * @example
   * reset the data to the initial state.
   * ```typescript
   * const session = await renderer.createSession(...);
   * await renderer.manipulateData(session, {
   *   action: 'reset',
   *   data: new Uint8Array(...),
   * });
   * ```
   * @example
   * merge the data to the current state.
   * ```typescript
   * const session = await renderer.createSession(...);
   * /// reset the data to the initial state
   * await renderer.manipulateData(session, data('reset'));
   * /// merge the data to the current state
   * await renderer.manipulateData(session, data('merge'));
   * /// incrementally merge the data again
   * await renderer.manipulateData(session, data('merge'));
   * ```
   */
  manipulateData(session: RenderSession, opts: ManipulateDataOptions): Promise<void>;

  /**
   * Run a function with a session, and the sesssion is only available during
   * the function call.
   *
   * the lifetime of session is quite bug-prone, so we current does not make it
   * longer live than the function call.
   * @param {function} fn - The function to run with a session.
   * @returns {Promise<T>} - The result of the function.
   * @example
   * run a function with an session with empty state.
   *
   * ```typescript
   * const res = await renderer.runWithSession(async session => {
   *   await renderer.manipulateData(session, data('reset'));
   *   return await renderer.renderToCanvas({
   *     renderSession: session,
   *     container: document.getElementById('container'),
   *     backgroundColor: '#ffffff',
   *   });
   * });
   * ```
   *
   * @example
   * run a function with an session with initial state.
   *
   * ```typescript
   * const res = await renderer.runWithSession({
   *   format: 'vector',
   *   artifactContent: new Uint8Array(...),
   * }, workWithSession(session));
   * ```
   *
   * @example
   * leak the life span of session (need typescript >= v5.2)
   *
   * ```typescript
   * class StackedSession {
   *   session: RenderSession;
   *   private resolve: (session: RenderSession) => void;
   *   [Symbol.dispose]() {
   *     this.resolve();
   *   }
   *   static async create() {
   *     return new Promise<StackedSession>(resolve => {
   *       const session = await renderer.runWithSession(session => {
   *       const stackedSession = new StackedSession();
   *       stackedSession.session = session;
   *       stackedSession.resolve = resolve;
   *       return stackedSession;
   *     });
   *   }
   * }
   *
   * {
   *   await using session = StackedSession.create();
   *   /// do something with session
   * }
   * ```
   */
  runWithSession<T>(fn: (session: RenderSession) => Promise<T>): Promise<T>;
  runWithSession<T>(
    options: CreateSessionOptions,
    fn: (session: RenderSession) => Promise<T>,
  ): Promise<T>;

  /**
   * alias to {@link TypstRenderer.renderToCanvas}, will remove in v0.5.0
   * @deprecated
   */
  render(options: RenderOptions<RenderToCanvasOptions>): Promise<void>;
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
export function createTypstRenderer(pdf: any): TypstRenderer {
  return new TypstRendererDriver(pdf);
}

export interface TypstSvgRenderer {
  init(options?: Partial<InitOptions>): Promise<void>;

  /**
   * Create a svg session.
   * @deprecated
   */
  createModule(b: Uint8Array): Promise<RenderSession>;

  /**
   * Render a Typst document to canvas.
   * @param session - The Typst document session that has been created by
   * TypstRenderer.
   * @param {HTMLElement} options - The options for rendering a Typst
   * document to specified container.
   * @deprecated
   */
  renderSvg(session: RenderSession, options: HTMLElement): Promise<unknown>;
}

export function createTypstSvgRenderer(): TypstSvgRenderer {
  return new TypstRendererDriver(undefined);
}

export function rendererBuildInfo(): any {
  return typst.renderer_build_info();
}

function randstr(prefix?: string): string {
  return Math.random()
    .toString(36)
    .replace('0.', prefix || '');
}

class TypstRendererDriver {
  renderer: typst.TypstRenderer;

  constructor(private pdf: any) {}

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.renderer = await buildComponent(options, gRendererModule, typst.TypstRendererBuilder, {});
  }

  loadGlyphPack(_pack: unknown): Promise<void> {
    // this.renderer.load_glyph_pack(pack);
    return Promise.resolve();
  }

  private createOptionsToRust(options: Partial<CreateSessionOptions>): typst.CreateSessionOptions {
    const rustOptions = new typst.CreateSessionOptions();

    if (options.format !== undefined) {
      rustOptions.format = options.format;
    }

    if (options.artifactContent !== undefined) {
      rustOptions.artifact_content = options.artifactContent;
    }

    return rustOptions;
  }

  loadPagesInfo(session: typst.RenderSession, options: RenderToCanvasOptions): PageInfo[] {
    const pages_info = session.pages_info;
    const pageInfos: PageInfo[] = [];
    const pageCount = pages_info.page_count;
    for (let i = 0; i < pageCount; i++) {
      const pageAst = pages_info.page(i);
      pageInfos.push({
        pageOffset: pageAst.page_off,
        width: pageAst.width_pt,
        height: pageAst.height_pt,
      });
    }

    return pageInfos;
  }

  renderCanvasInSession(
    session: RenderSession,
    canvas: CanvasRenderingContext2D,
    options?: RenderPageOptions,
  ): Promise<RenderPageResult> {
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
    container: HTMLElement,
    canvasList: HTMLCanvasElement[],
    options: RenderToCanvasOptions,
  ): Promise<RenderPageResult[]> {
    const pages_info = session.pages_info;
    const page_count = pages_info.page_count;

    const doRender = async (i: number, page_off: number) => {
      const canvas = canvasList[i];
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        throw new Error('canvas context is null');
      }
      return await this.renderCanvasInSession(session, ctx, {
        page_off,
      });
    };

    return this.inAnimationFrame(async () => {
      const t = performance.now();

      const textContentList = (
        await Promise.all(
          //   canvasList.map(async (canvas, i) => {
          //     await this.renderImageInSession(session, {
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
              const results: RenderPageResult[] = [];
              for (let i = 0; i < page_count; i++) {
                results.push(await doRender(i, i));
              }

              return results;
            })(),
          ],
        )
      )[0];

      const t3 = performance.now();

      console.log(`display layer used: render = ${(t3 - t).toFixed(1)}ms`);

      return textContentList;
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
    container: HTMLElement,
    layerList: HTMLDivElement[],
    textSourceList: any[],
  ) {
    renderTextLayer(this.pdf, container, view.pageInfos, layerList, textSourceList);
  }

  private renderAnnotationLayer(
    _session: typst.RenderSession,
    view: RenderView,
    _container: HTMLElement,
    layerList: HTMLDivElement[],
    annotationSourceList: AnnotationList[],
  ) {
    const pageInfos = view.pageInfos;

    const t2 = performance.now();

    const renderOne = (layer: HTMLDivElement, i: number) => {
      const page_info = pageInfos[i];
      if (!page_info) {
        console.error('page not found for', i);
        return;
      }
      const width_pt = page_info.width;
      const height_pt = page_info.height;

      layer.innerHTML = '';
      for (const lnk of annotationSourceList[i].links) {
        const annotationBox = document.createElement('div');
        const x = (lnk.annotation_box.transform[4] / width_pt) * 100;
        const y = (lnk.annotation_box.transform[5] / height_pt) * 100;
        const skewY = lnk.annotation_box.transform[1];
        const skewX = lnk.annotation_box.transform[2];
        annotationBox.className = 'typst-annotation';
        annotationBox.style.width = `${(lnk.annotation_box.width / width_pt) * 100}%`;
        annotationBox.style.height = `${(lnk.annotation_box.height / height_pt) * 100}%`;
        annotationBox.style.left = `${x}%`;
        annotationBox.style.top = `${y}%`;
        annotationBox.style.transform = `matrix(1, ${skewY}, ${skewX}, 1, 0, 0)`;

        switch (lnk.action.t) {
          case 'Url': {
            const a = document.createElement('a');
            a.href = lnk.action.v.url;
            a.target = '_blank';
            a.appendChild(annotationBox);
            layer.appendChild(a);
            break;
          }
          case 'GoTo': {
            const destPoint = document.createElement('div');
            destPoint.className = 'typst-annotation';
            const destX = (lnk.action.v.x / width_pt) * 100;
            const destY = (lnk.action.v.y / height_pt) * 100;
            destPoint.style.left = `${destX}%`;
            destPoint.style.top = `${destY}%`;
            const destId = randstr('lnk-');
            destPoint.id = destId;

            // todo: imcomplete rendering should load these pages before appendChild
            const destLayer = layerList[lnk.action.v.page_ref - 1];
            destLayer.appendChild(destPoint);

            const a = document.createElement('a');
            a.href = `#${destId}`;
            a.appendChild(annotationBox);
            layer.appendChild(a);
            break;
          }
          default:
            console.warn('unknown action', lnk);
            break;
        }
      }
    };

    layerList.forEach(renderOne);
    const t3 = performance.now();
    console.log(`annotation layer used: render = ${(t3 - t2).toFixed(1)}ms`);
  }

  async render(options: RenderOptions<RenderToCanvasOptions>): Promise<void> {
    if ('format' in options) {
      if (options.format !== 'vector') {
        const artifactFormats = ['serde_json', 'js', 'ir'] as const;
        if (artifactFormats.includes(options.format as any)) {
          // deprecated
          throw new Error(`deprecated format ${options.format}, please use vector format`);
        }
      }
    }

    return this.renderToCanvas(options);
  }

  async renderToCanvas(options: RenderOptions<RenderToCanvasOptions>): Promise<void> {
    let session: typst.RenderSession;
    let renderPageResults: RenderPageResult[];
    const mountContainer = options.container;
    mountContainer.style.visibility = 'hidden';

    const doRenderDisplayLayer = async (
      canvasList: HTMLCanvasElement[],
      resetLayout: () => void,
    ) => {
      try {
        renderPageResults = await this.renderDisplayLayer(
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

      if (options.pixelPerPt !== undefined && options.pixelPerPt <= 0) {
        throw new Error(
          'Invalid typst.RenderOptions.pixelPerPt, should be a positive number ' +
            options.pixelPerPt,
        );
      }

      let backgroundColor = options.backgroundColor;
      if (backgroundColor !== undefined) {
        if (!/^#[0-9a-f]{6}$/.test(backgroundColor)) {
          throw new Error(
            'Invalid typst.backgroundColor color for matching ^#?[0-9a-f]{6}$ ' + backgroundColor,
          );
        }
      }

      session.pixel_per_pt = options.pixelPerPt ?? 3;
      session.background_color = backgroundColor ?? '#ffffff';

      const t = performance.now();

      const pageView = new RenderView(
        this.loadPagesInfo(session, options),
        mountContainer,
        options,
      );
      const t2 = performance.now();

      console.log(`layer used: retieve = ${(t2 - t).toFixed(1)}ms`);

      await doRenderDisplayLayer(pageView.canvasList, () => pageView.resetLayout());
      this.renderTextLayer(
        session,
        pageView,
        mountContainer,
        pageView.textLayerList,
        renderPageResults.map(r => r.textContent),
      );
      this.renderAnnotationLayer(
        session,
        pageView,
        mountContainer,
        pageView.annotationLayerList,
        renderPageResults.map(r => r.annotationList),
      );

      return;
    });
  }

  createModule(b?: Uint8Array): Promise<RenderSession> {
    return Promise.resolve(
      this.renderer.create_session(
        this.createOptionsToRust({
          format: 'vector',
          artifactContent: b,
        }),
      ),
    );
  }

  renderSvg(session: RenderSession, container: HTMLElement): Promise<void> {
    return Promise.resolve(this.renderer.render_svg(session as typst.RenderSession, container));
  }

  renderToSvg(options: RenderOptions<RenderToSvgOptions>): Promise<void> {
    return this.withinOptionSession(options, async sessionRef => {
      return this.renderSvg(sessionRef, options.container);
    });
  }

  manipulateData(session: RenderSession, opts: ManipulateDataOptions): Promise<void> {
    return Promise.resolve(
      this.renderer.manipulate_data(
        session as typst.RenderSession,
        opts.action ?? 'reset',
        opts.data,
      ),
    );
  }

  private withinOptionSession<T>(
    options: RenderToCanvasOptions | CreateSessionOptions,
    fn: (session: typst.RenderSession) => Promise<T>,
  ): Promise<T> {
    function isRenderByContentOption(
      options: RenderToCanvasOptions | CreateSessionOptions,
    ): options is CreateSessionOptions {
      return 'artifactContent' in options;
    }

    if ('renderSession' in options) {
      return fn(options.renderSession as typst.RenderSession);
    }

    if (isRenderByContentOption(options)) {
      // todo: remove any
      return this.runWithSession(options as any, fn as any);
    }

    throw new Error(
      'Invalid render options, should be one of RenderByContentOptions|RenderBySessionOptions',
    );
  }

  async runWithSession<T>(fn: (session: RenderSession) => Promise<T>): Promise<T>;
  runWithSession<T>(
    options: CreateSessionOptions,
    fn: (session: RenderSession) => Promise<T>,
  ): Promise<T>;
  async runWithSession<T>(arg1: any, arg2?: any): Promise<T> {
    let options: Partial<CreateSessionOptions> = arg1;
    let fn: (session: RenderSession) => Promise<T> = arg2;

    if (!arg2) {
      options = {};
      fn = arg1;
    }

    // const t = performance.now();
    const session = this.renderer.create_session(/* moved */ this.createOptionsToRust(options));
    // const t3 = performance.now();

    // console.log(`create session used: render = ${(t3 - t).toFixed(1)}ms`);
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

type TransformMatrix = [number, number, number, number, number, number];

interface AnnotationBox {
  height: number;
  width: number;
  page_ref: number;
  transform: TransformMatrix;
}

interface UrlLinkAction {
  t: 'Url';
  v: {
    url: string;
  };
}

interface GoToLinkAction {
  t: 'GoTo';
  v: {
    page_ref: number;
    x: number;
    y: number;
  };
}

type LinkAction = UrlLinkAction | GoToLinkAction;

interface LinkAnnotation {
  annotation_box: AnnotationBox;
  action: LinkAction;
}

interface AnnotationList {
  links: LinkAnnotation[];
}
