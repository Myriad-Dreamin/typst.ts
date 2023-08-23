// @ts-ignore
import typstInit, * as typst from '@myriaddreamin/typst-ts-renderer';

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

// action
// :
// t
// :
// "Url"
// v
// :
// {uri: 'https://github.com/johanvx/typst-undergradmath'}
// [[Prototype]]
// :
// Object
// annotation_box
// :
// height
// :
// 7.740234375
// page_ref
// :
// 0
// transform
// :
// (6) [4.5, 0, 0, 4.5, 395.22845458984375, 162]
// width
// :
// 147.515625

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

export interface RenderPageResult {
  textContent: any;
  annotationList: AnnotationList;
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
export function createTypstRenderer(pdf: any): TypstRenderer {
  return new TypstRendererDriver(pdf);
}

export interface TypstSvgRenderer {
  init(options?: Partial<InitOptions>): Promise<void>;
  createModule(b: Uint8Array): Promise<unknown>;
}

export function createTypstSvgRenderer(): TypstSvgRenderer {
  return new TypstRendererSvgDriver();
}

export function rendererBuildInfo(): any {
  return typst.renderer_build_info();
}

function randstr(prefix?: string): string {
  return Math.random()
    .toString(36)
    .replace('0.', prefix || '');
}

class TypstRendererSvgDriver {
  renderer: typst.TypstRenderer;

  constructor() {}

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.renderer = await buildComponent(options, gRendererModule, typst.TypstRendererBuilder, {});
  }

  createModule(b?: Uint8Array): Promise<unknown> {
    return new Promise(resolve => {
      resolve(b ? this.renderer.create_svg_session(b) : this.renderer.create_empty_svg_session());
    });
  }

  renderSvg(session: unknown, container: HTMLDivElement): Promise<unknown> {
    return new Promise(resolve => {
      resolve(this.renderer.render_svg(session as typst.SvgSession, container));
    });
  }
}

class TypstRendererDriver {
  renderer: typst.TypstRenderer;

  constructor(private pdf: any) {}

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
  ): Promise<[RenderResult, RenderPageResult[]]> {
    const pages_info = session.pages_info;
    const page_count = pages_info.page_count;

    /// render each page
    let renderResult = undefined as unknown as RenderResult;

    const doRender = async (i: number, page_off: number) => {
      const canvas = canvasList[i];
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        throw new Error('canvas context is null');
      }
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

  private renderAnnotationLayer(
    _session: typst.RenderSession,
    view: RenderView,
    _container: HTMLDivElement,
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

  async render(options: RenderOptions): Promise<RenderResult> {
    let session: typst.RenderSession;
    let renderResult: RenderResult;
    let renderPageResults: RenderPageResult[];
    const mountContainer = options.container;
    mountContainer.style.visibility = 'hidden';

    const doRenderDisplayLayer = async (
      canvasList: HTMLCanvasElement[],
      resetLayout: () => void,
    ) => {
      try {
        [renderResult, renderPageResults] = await this.renderDisplayLayer(
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
