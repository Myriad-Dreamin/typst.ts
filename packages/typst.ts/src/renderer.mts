// @ts-ignore
import type * as typst from '@myriaddreamin/typst-ts-renderer';

import type { InitOptions } from './options.init.mjs';
import { PageInfo, RenderCanvasResult, TypstDefaultParams, kObject } from './internal.types.mjs';
import {
  CreateSessionOptions,
  RenderToCanvasOptions,
  RenderOptions,
  RenderCanvasOptions,
  RenderToSvgOptions,
  ManipulateDataOptions,
  RenderSvgOptions,
  RenderInSessionOptions,
  MountDomOptions,
  OffscreenRenderCanvasOptions,
} from './options.render.mjs';
import { RenderView } from './render/canvas/view.mjs';
import { LazyWasmModule } from './wasm.mjs';
import { buildComponent } from './init.mjs';
import { TypstDomDocument } from './dom.mjs';

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

type ContextedRenderOptions<T> = T | RenderOptions<T>;

/**
 * The session of a Typst document.
 * @property {string} backgroundColor - The background color of the Typst
 * document.
 * @property {number} pixelPerPt - The pixel per point scale up the image.
 *
 */
export class RenderSession {
  /**
   * @internal
   */
  public [kObject]: typst.RenderSession;

  /**
   * @internal
   */
  constructor(
    /**
     * @internal
     */
    private plugin: TypstRenderer,
    o: typst.RenderSession,
  ) {
    this[kObject] = o;
  }

  /**
   * @deprecated set in {@link RenderToCanvasOptions} instead
   *
   * Set the background color of the Typst document.
   * @param {string} t - The background color in format of `^#?[0-9a-f]{6}$`
   *
   * Note: Default to `#ffffff`.
   *
   * Note: Only available in canvas rendering mode.
   */
  set backgroundColor(t: string | undefined) {
    if (t !== undefined) {
      this[kObject].background_color = t;
    }
  }

  /**
   * Get the background color of the Typst document.
   *
   * Note: Default to `#ffffff`.
   *
   * Note: Only available in canvas rendering mode.
   */
  get backgroundColor(): string | undefined {
    return this[kObject].background_color;
  }

  /**
   * Set the pixel per point scale up the canvas panel.
   *
   * Note: Default to `3`.
   *
   * Note: Only available in canvas rendering mode.
   */
  set pixelPerPt(t: number | undefined) {
    if (t !== undefined) {
      this[kObject].pixel_per_pt = t;
    }
  }

  /**
   * @deprecated set in {@link RenderToCanvasOptions} instead
   *
   * Get the pixel per point scale up the canvas panel.
   *
   * Note: Default to `3`.
   *
   * Note: Only available in canvas rendering mode.
   */
  get pixelPerPt(): number | undefined {
    return this[kObject].pixel_per_pt;
  }

  /**
   * Reset state
   */
  reset(): void {
    this.plugin.resetSession(this);
  }

  /**
   * @deprecated
   * use {@link docWidth} instead
   */
  get doc_width(): number {
    return this[kObject].doc_width;
  }

  get docWidth(): number {
    return this[kObject].doc_width;
  }

  /**
   * @deprecated
   * use {@link docHeight} instead
   */
  get doc_height(): number {
    return this[kObject].doc_height;
  }

  get docHeight(): number {
    return this[kObject].doc_height;
  }

  retrievePagesInfo(): PageInfo[] {
    const pages_info = this[kObject].pages_info;
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

  getSourceLoc(path: Uint32Array): string | undefined {
    return (this[kObject] as typst.RenderSession).source_span(path);
  }

  /**
   * See {@link TypstRenderer#renderSvg} for more details.
   */
  renderSvg(options: ContextedRenderOptions<RenderSvgOptions>): Promise<string> {
    return this.plugin.renderSvg({
      renderSession: this,
      ...options,
    });
  }

  /**
   * See {@link TypstRenderer#renderToSvg} for more details.
   */
  renderToSvg(options: ContextedRenderOptions<RenderToSvgOptions>): Promise<boolean> {
    return this.plugin.renderToSvg({
      renderSession: this,
      ...options,
    });
  }

  /**
   * See {@link TypstRenderer#renderCanvas} for more details.
   */
  renderCanvas(options: ContextedRenderOptions<RenderCanvasOptions>): Promise<RenderCanvasResult> {
    return this.plugin.renderCanvas({
      renderSession: this,
      ...options,
    });
  }

  /**
   * See {@link TypstRenderer#manipulateData} for more details.
   */
  manipulateData(opts: ManipulateDataOptions) {
    this.plugin.manipulateData({
      renderSession: this,
      ...opts,
    });
  }

  /**
   * See {@link TypstRenderer#renderSvgDiff} for more details.
   */
  renderSvgDiff(opts: RenderSvgOptions): string {
    return this.plugin.renderSvgDiff({
      renderSession: this,
      ...opts,
    });
  }

  /**
   * @deprecated
   * use {@link getSourceLoc} instead
   */
  get_source_loc(path: Uint32Array): string | undefined {
    return (this[kObject] as typst.RenderSession).source_span(path);
  }

  /**
   * @deprecated
   * use {@link renderSvgDiff} instead
   */
  render_in_window(rect_lo_x: number, rect_lo_y: number, rect_hi_x: number, rect_hi_y: number) {
    return this[kObject].render_in_window(rect_lo_x, rect_lo_y, rect_hi_x, rect_hi_y);
  }

  /**
   * @deprecated
   * use {@link manipulateData} instead
   */
  merge_delta(data: Uint8Array) {
    this.plugin.manipulateData({
      renderSession: this,
      action: 'merge',
      data,
    });
  }
}

/**
 * @internal
 */
const enum ManageStatus {
  Delete,
  New,
  Update,
}

export class TypstWorker {
  /**
   * @internal
   */
  public [kObject]: typst.TypstWorker;

  /**
   * @internal
   */
  constructor(
    /**
     * @internal
     */
    private plugin: TypstRendererDriver,
    o: typst.TypstWorker,
  ) {
    this[kObject] = o;
  }

  /**
   * See {@link TypstRenderer#manipulateData} for more details.
   */
  manipulateData(action: string, data: Uint8Array): Promise<void> {
    return this[kObject].manipulate_data(action, data);
  }

  /**
   * @internal
   */
  managedCanvasElemList = new Map<string, [ManageStatus, OffscreenRenderCanvasOptions]>();
  /**
   * @internal
   */
  canvasCounter = Math.random();
  /**
   * You must submit all canvas in pages to ensure synchronization with the background worker
   *
   * See {@link TypstRenderer#renderCanvas} for more details.
   */
  renderCanvas(canvasElemList: OffscreenRenderCanvasOptions[]): Promise<RenderCanvasResult[]> {
    const m = this.managedCanvasElemList;
    for (const [_, elem] of m) {
      elem[0] = ManageStatus.Delete;
    }

    for (const elem of canvasElemList) {
      const canvas = elem.canvas;
      let elemId = canvas.dataset.manageId;
      let action = ManageStatus.Update;
      if (!elemId) {
        elemId = this.canvasCounter.toFixed(5);
        this.canvasCounter += 1;
        canvas.dataset.manageId = elemId;
        action = ManageStatus.New;
      }

      let prev = m.get(elemId);
      if (prev && prev[0] !== ManageStatus.Delete) {
        throw new Error('cannot update a canvas for two times in batch');
      }

      m.set(elemId, [action, { ...elem }]);
    }

    const entries = Array.from(m.entries());
    const actions = new Uint8Array(entries.length);
    const elements = new Array(entries.length);
    const options = entries.map(([key, [action, elem]], index) => {
      if (!action) {
        m.delete(key);
      }

      actions[index] = action;
      elements[index] = elem.canvas;
      return this.plugin.canvasOptionsToRust(elem);
    });

    return this[kObject].render_canvas(actions, elements, options);
  }

  async retrievePagesInfo(): Promise<PageInfo[]> {
    const pages_info = await this[kObject].get_pages_info();
    console.log(pages_info);
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
}

/**
 * @deprecated
 * use {@link TypstRenderer} instead
 */
export interface TypstSvgRenderer {
  /**
   * Render a Typst document to svg.
   * @param {RenderOptions<RenderToSvgOptions>} options - The options for
   * rendering a Typst document to specified container.
   * @returns {void} - The result of rendering a Typst document.
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
  renderToSvg(options: RenderOptions<RenderToSvgOptions>): Promise<boolean>;
}


/**
 * The interface of Typst renderer.
 */
export interface TypstRenderer extends TypstSvgRenderer {
  /**
   * Initialize the typst renderer.
   * @param {Partial<InitOptions>} options - The options for initializing the
   * typst renderer.
   */
  init(options?: Partial<InitOptions>): Promise<void>;

  /**
   * Load a glyph pack for all of the Typst documents to render.
   * Note: this function is still under development.
   * @param pack
   */
  loadGlyphPack(pack: unknown): Promise<void>;

  /**
   * Reset state
   */
  resetSession(session: RenderSession): void;

  /**
   * Retrieve page information of current selected document
   */
  retrievePagesInfoFromSession(session: RenderSession): PageInfo[];

  /**
   * Render a Typst document to canvas.
   */
  renderCanvas(options: RenderOptions<RenderCanvasOptions>): Promise<RenderCanvasResult>;

  /**
   * Render a Typst document to canvas.
   * @param {RenderOptions<RenderToCanvasOptions>} options - The options for
   * rendering a Typst document to specified container.
   * @returns {void} - The result of rendering a Typst document.
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
   * Render a Typst document to (non-incremental) svg string.
   * @param {RenderOptions<RenderSvgOptions>} options - The options for
   * rendering a Typst document to specified container.
   * @returns {string} - The rendered content.
   * @example
   * ```typescript
   * let fetchDoc = (path) => fetch(path).then(
   *   response => new Uint8Array(response.arrayBuffer()))
   * const svg = renderer.renderSvg({
   *   artifactContent: await fetchDoc('typst-main.sir.in'),
   * });
   * ```
   */
  renderSvg(options: RenderOptions<RenderSvgOptions>): Promise<string>;

  /**
   * Render a Typst document to svg.
   * @param {RenderOptions<RenderToSvgOptions>} options - The options for
   * rendering a Typst document to specified container.
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
  renderToSvg(options: RenderOptions<RenderToSvgOptions>): Promise<boolean>;

  /**
   * Return selected current svg
   */
  getCustomV1(options: RenderInSessionOptions<{}>): Promise<any>;

  /**
   * experimental
   */
  renderSvgDiff(options: RenderInSessionOptions<RenderSvgOptions>): string;

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
  manipulateData(opts: RenderInSessionOptions<ManipulateDataOptions>): void;

  /**
   * Run a function with a session, and the session is only available during
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

  createWorkerV0(worker: Worker): Promise<TypstWorker>;

  renderDom(options: RenderInSessionOptions<MountDomOptions>): Promise<TypstDomDocument>;

  /**
   * alias to {@link TypstRenderer#renderToCanvas}, will remove in v0.5.0
   * @deprecated
   * use {@link renderToCanvas} instead
   */
  render(options: RenderOptions<RenderToCanvasOptions>): Promise<void>;
}

const gRendererModule = (module: typeof typst) =>
  new LazyWasmModule(async (bin?: any) => {
    return await module.default(bin);
  });

/**
 * create a Typst renderer.
 * @returns {TypstRenderer} - The Typst renderer.
 * @example
 * ```typescript
 * import { createTypstRenderer } from 'typst';
 * const renderer = createTypstRenderer();
 * await renderer.init();
 * await renderer.render({
 *   container: document.getElementById('container'),
 *   artifactContent: '{ ... }',
 * });
 * ```
 */
export function createTypstRenderer(): TypstRenderer {
  return new TypstRendererDriver();
}

export async function rendererBuildInfo(): Promise<any> {
  const renderModule = await import('@myriaddreamin/typst-ts-renderer');
  return renderModule.renderer_build_info();
}

let warnOnceCanvasSet = true;

/** @internal */
export class TypstRendererDriver {
  renderer: typst.TypstRenderer;
  rendererJs: typeof typst;

  constructor() { }

  async init(options?: Partial<InitOptions>): Promise<void> {
    this.rendererJs = await (options?.getWrapper?.() || import('@myriaddreamin/typst-ts-renderer'));
    const TypstRendererBuilder = this.rendererJs.TypstRendererBuilder;
    this.renderer = await buildComponent(
      options,
      gRendererModule(this.rendererJs),
      TypstRendererBuilder,
      {},
    );
  }

  loadGlyphPack(_pack: unknown): Promise<void> {
    // this.renderer.load_glyph_pack(pack);
    return Promise.resolve();
  }

  private createOptionsToRust(options: Partial<CreateSessionOptions>): typst.CreateSessionOptions {
    const rustOptions = new this.rendererJs.CreateSessionOptions();

    if (options.format !== undefined) {
      rustOptions.format = options.format;
    }

    if (options.artifactContent !== undefined) {
      rustOptions.artifact_content = options.artifactContent;
    }

    return rustOptions;
  }

  canvasOptionsToRust(options: RenderCanvasOptions): typst.RenderPageImageOptions {
    const rustOptions = new this.rendererJs.RenderPageImageOptions();
    if (options.pageOffset === undefined) {
      throw new Error('pageOffset is required in reflexo v0.5.0');
    } else {
      rustOptions.page_off = options.pageOffset;
    }
    if (options.cacheKey !== undefined) {
      rustOptions.cache_key = options.cacheKey;
    }
    if (options.backgroundColor !== undefined) {
      rustOptions.background_color = options.backgroundColor;
    }
    if (options.pixelPerPt !== undefined) {
      rustOptions.pixel_per_pt = options.pixelPerPt;
    }
    if (options.dataSelection !== undefined) {
      let encoded = 0;
      if (options.dataSelection.body) {
        encoded |= 1 << 0;
      } else if (options.canvas && warnOnceCanvasSet) {
        warnOnceCanvasSet = false;
        console.warn('dataSelection.body is not set but providing canvas for body');
      }
      if (options.dataSelection.text || options.dataSelection.annotation) {
        console.error('dataSelection.text and dataSelection.annotation are deprecated');
      }
      if (options.dataSelection.semantics) {
        encoded |= 1 << 3;
      }
      rustOptions.data_selection = encoded;
    }
    return rustOptions;
  }

  retrievePagesInfoFromSession(session: RenderSession): PageInfo[] {
    return session.retrievePagesInfo();
  }

  /**
   * Render a Typst document to canvas.
   */
  renderCanvas(options: RenderOptions<RenderCanvasOptions>): Promise<RenderCanvasResult> {
    return this.withinOptionSession(options, async sessionRef => {
      return this.renderer.render_page_to_canvas(
        sessionRef[kObject],
        options.canvas || undefined,
        this.canvasOptionsToRust(options),
      );
    });
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
    session: RenderSession,
    canvasList: HTMLCanvasElement[],
    options: RenderToCanvasOptions,
  ): Promise<RenderCanvasResult[]> {
    const pages_info = session[kObject].pages_info;
    const page_count = pages_info.page_count;

    const doRender = async (i: number, page_off: number) => {
      const canvas = canvasList[i];
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        throw new Error('canvas context is null');
      }
      return await this.renderCanvas({
        ...options,
        canvas: ctx,
        renderSession: session,
        pageOffset: page_off,
      });
    };

    const t = performance.now();
    const textContentList = await (async () => {
      const results: RenderCanvasResult[] = [];
      for (let i = 0; i < page_count; i++) {
        results.push(await this.inAnimationFrame(() => doRender(i, i)));
      }

      return results;
    })();
    const t2 = performance.now();

    console.log(`display layer used: render = ${(t2 - t).toFixed(1)}ms`);

    return textContentList;
  }

  private renderTextLayer(layerList: HTMLDivElement[], textSourceList: RenderCanvasResult[]) {
    const t2 = performance.now();
    layerList.forEach((layer, i) => {
      layer.innerHTML = textSourceList[i].htmlSemantics[0];
    });
    const t3 = performance.now();
    console.log(`text layer used: render = ${(t3 - t2).toFixed(1)}ms`);
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

  async renderDom(options: RenderInSessionOptions<MountDomOptions>): Promise<TypstDomDocument> {
    if ('format' in options) {
      if (options.format !== 'vector') {
        const artifactFormats = ['serde_json', 'js', 'ir'] as const;
        if (artifactFormats.includes(options.format as any)) {
          // deprecated
          throw new Error(`deprecated format ${options.format}, please use vector format`);
        }
      }
    }

    return this.withinOptionSession(options, async sessionRef => {
      const t = new TypstDomDocument({
        ...options,
        renderMode: 'dom',
        hookedElem: options.container,
        kModule: sessionRef,
        renderer: this,
      });
      t;
      await t.impl.mountDom(options.pixelPerPt);
      return t;
    });
  }

  async renderToCanvas(options: RenderOptions<RenderToCanvasOptions>): Promise<void> {
    let session: RenderSession;
    let renderPageResults: RenderCanvasResult[];
    const mountContainer = options.container;
    mountContainer.style.visibility = 'hidden';

    const doRenderDisplayLayer = async (
      canvasList: HTMLCanvasElement[],
      resetLayout: () => void,
    ) => {
      try {
        renderPageResults = await this.renderDisplayLayer(session, canvasList, options);
        resetLayout();
      } finally {
        mountContainer.style.visibility = 'visible';
      }
    };

    return this.withinOptionSession(options, async sessionRef => {
      session = sessionRef;
      if (session[kObject].pages_info.page_count === 0) {
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

      session.pixelPerPt = options.pixelPerPt ?? TypstDefaultParams.PIXEL_PER_PT;
      session.backgroundColor = backgroundColor ?? '#ffffff';

      const t = performance.now();

      const pageView = new RenderView(
        this.retrievePagesInfoFromSession(session),
        mountContainer,
        options,
      );
      const t2 = performance.now();

      console.log(`layer used: retrieve = ${(t2 - t).toFixed(1)}ms`);

      await doRenderDisplayLayer(pageView.canvasList, () => pageView.resetLayout());
      this.renderTextLayer(pageView.textLayerList, renderPageResults);

      return;
    });
  }

  createModule(b?: Uint8Array): Promise<RenderSession> {
    return Promise.resolve(
      new RenderSession(
        this,
        this.renderer.create_session(
          b &&
          this.createOptionsToRust({
            format: 'vector',
            artifactContent: b,
          }),
        ),
      ),
    );
  }

  async createWorkerV0(worker: Worker) {
    return new TypstWorker(this, await this.renderer.create_worker(worker));
  }

  workerBridge() {
    return this.renderer.create_worker_bridge();
  }

  renderSvg(options: RenderOptions<RenderSvgOptions>, container?: any): Promise<string> {
    if (options instanceof RenderSession || container) {
      throw new Error('removed api, please use renderToSvg({ renderSession, container }) instead');
    }

    return this.withinOptionSession(options, async sessionRef => {
      let parts: number | undefined = undefined;
      if (options.data_selection) {
        parts = 0;
        if (options.data_selection.body) {
          parts |= 1 << 0;
        }
        if (options.data_selection.defs) {
          parts |= 1 << 1;
        }
        if (options.data_selection.css) {
          parts |= 1 << 2;
        }
        if (options.data_selection.js) {
          parts |= 1 << 3;
        }
      }

      return Promise.resolve(this.renderer.svg_data(sessionRef[kObject], parts));
    });
  }

  renderSvgDiff(options: RenderInSessionOptions<RenderSvgOptions>): string {
    if (!options.window) {
      return this.renderer.render_svg_diff(
        (options.renderSession as any)[kObject],
        0,
        0,
        1e33,
        1e33,
      );
    }

    return this.renderer.render_svg_diff(
      (options.renderSession as any)[kObject],
      options.window.lo.x,
      options.window.lo.y,
      options.window.hi.x,
      options.window.hi.y,
    );
  }

  renderToSvg(options: RenderOptions<RenderToSvgOptions>): Promise<boolean> {
    return this.withinOptionSession(options, async sessionRef => {
      return Promise.resolve(this.renderer.render_svg(sessionRef[kObject], options.container));
    });
  }

  getCustomV1(options: RenderInSessionOptions<{}>): Promise<any> {
    return Promise.resolve(this.renderer.get_customs(options.renderSession[kObject]));
  }

  resetSession(session: RenderSession): void {
    return this.renderer.reset(session[kObject]);
  }

  manipulateData(opts: RenderInSessionOptions<ManipulateDataOptions>): void {
    return this.renderer.manipulate_data(
      (opts.renderSession as any)[kObject] as typst.RenderSession,
      opts.action ?? 'reset',
      opts.data,
    );
  }

  private withinOptionSession<T>(
    options: RenderOptions<any>,
    fn: (session: RenderSession) => Promise<T>,
  ): Promise<T> {
    function isRenderByContentOption(options: RenderOptions<any>): options is CreateSessionOptions {
      return 'artifactContent' in options;
    }

    if ('renderSession' in options) {
      return fn(options.renderSession as RenderSession);
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
    let options: Partial<CreateSessionOptions> | undefined = arg1;
    let fn: (session: RenderSession) => Promise<T> = arg2;

    if (!arg2) {
      options = undefined;
      fn = arg1;
    }

    const session = this.renderer.create_session(
      /* moved */ options && this.createOptionsToRust(options),
    );
    try {
      const res = await fn(new RenderSession(this, session));
      session.free();
      return res;
    } catch (e) {
      session.free();
      throw e;
    }
  }
}
