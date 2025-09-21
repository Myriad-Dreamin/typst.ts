import type { IncrDomDocClient } from '@myriaddreamin/typst-ts-renderer';
import { Rect, kObject } from './internal.types.mjs';
import { TypstRenderer, TypstRendererDriver } from './renderer.mjs';
import {
  GConstructor,
  TypstDocumentContext,
  composeDoc,
  provideDoc,
} from './contrib/dom/typst-doc.mjs';
import { TypstCancellationToken } from './contrib/dom/typst-cancel.mjs';

const animationFrame = () => new Promise(resolve => requestAnimationFrame(resolve));

class DomPage {
  dispose() { }
}

const enum TrackMode {
  Doc,
  Pages,
}

const enum RepaintStage {
  Layout = 0,
  Svg = 1,
  Semantics = 2,
  PrepareCanvas = 3,
  Canvas = 4,
}

export interface ITypstDomDocument {
  mountDom(pixelPerPt: number | undefined): Promise<void>;
}

export interface InitDomDocArgs {
  renderer: TypstRenderer;
  domScale?: number;
}

interface RenderTask {
  cancel(): Promise<void>;
}

export function provideDomDoc<TBase extends GConstructor<TypstDocumentContext<InitDomDocArgs>>>(
  Base: TBase,
): TBase & GConstructor<ITypstDomDocument> {
  return class DomDocument extends Base {
    /// The template element for creating DOM by string.
    tmpl = document.createElement('template');
    /// The stub element for replacing an invisible element.
    stub = this.createElement('<stub></stub>');
    /// Typescript side of lib.
    plugin: TypstRendererDriver;
    /// Rust side of kernel.
    docKernel: IncrDomDocClient;
    /// The element to track.
    resourceHeader: SVGElement = undefined!;
    /// Expected exact state of the current DOM.
    /// Initially it is empty meaning no any page is rendered.
    pages: DomPage[] = [];
    /// The virtual scale of the document.
    domScale = 1;
    /// Track mode.
    track_mode: TrackMode = TrackMode.Doc;
    /// Current executing task.
    current_task?: RenderTask = undefined;
    /// The currently maintained viewport.
    viewport: Rect;
    constructor(...args: any[]) {
      super(...args);
      this.registerMode('dom');
      this.disposeList.push(() => {
        this.dispose();
      });
      this.plugin = this.opts.renderer as any as TypstRendererDriver;
      if (this.opts.domScale !== undefined) {
        if (this.opts.domScale <= 0) {
          throw new Error('domScale must be positive');
        }
        this.domScale = this.opts.domScale;
      }
    }

    dispose() {
      for (const page of this.pages) {
        page.dispose();
      }

      if (this.docKernel) {
        this.docKernel.free();
      }
    }

    createElement(tmpl: string) {
      this.tmpl.innerHTML = tmpl;
      return this.tmpl.content.firstElementChild;
    }

    async mountDom(pixelPerPt: number | undefined) {
      // console.log('mountDom', pixelPerPt);

      if (this.docKernel) {
        throw new Error('already mounted');
      }

      // create typst-svg-resources by string
      this.hookedElem.innerHTML = `<svg class="typst-svg-resources" viewBox="0 0 0 0" width="0" height="0" style="opacity: 0; position: absolute;"></svg>`;
      this.resourceHeader = this.hookedElem.querySelector('.typst-svg-resources')!;

      this.docKernel = await this.plugin.renderer.mount_dom(this.kModule[kObject], this.hookedElem);

      this.docKernel.bind_functions({
        populateGlyphs: (data: string) => {
          let svg = this.createElement(data)!;
          // console.log('populateGlyphs', svg);
          let content = svg.firstElementChild!;
          this.resourceHeader.append(content);
        },
      });
    }

    async cancelAnyway$dom() {
      // console.log('cancelAnyway$dom');
      if (this.current_task) {
        const task = this.current_task;
        this.current_task = undefined;
        await task.cancel();
      }
    }

    retrieveDOMPages() {
      return Array.from(this.hookedElem.querySelectorAll('.typst-dom-page'));
    }

    // doesn't need to postRender
    postRender$dom() { }

    // doesn't need to rescale
    rescale$dom() { }

    getDomViewport(
      cachedWindow: Pick<Window, 'innerHeight' | 'innerWidth'>,
      cachedBoundingRect: Pick<DOMRect, 'left' | 'top' | 'right'>,
    ) {
      const left = cachedBoundingRect.left;
      const top = -cachedBoundingRect.top;
      const right = cachedBoundingRect.right;
      const bottom = cachedWindow.innerHeight - cachedBoundingRect.top;
      const rect = {
        x: 0,
        y: top / this.domScale,
        width: Math.max(right - left, 0) / this.domScale,
        height: Math.max(bottom - top, 0) / this.domScale,
      };
      if (rect.width <= 0 || rect.height <= 0) {
        rect.x = rect.y = rect.width = rect.height = 0;
      }
      // console.log('ccc', basePos, appPos, rect);
      return rect;
    }

    // fast mode
    async rerender$dom() {
      const domState = this.retrieveDOMState();

      // const l = domState.boundingRect.left;
      const { x, y, width, height } = this.getDomViewport(domState.window, domState.boundingRect);

      let dirty = await this.docKernel.relayout(x, y, width, height);
      if (!dirty) {
        return;
      }

      const cancel = new TypstCancellationToken();
      this.doRender$dom(cancel);
      this.current_task = cancel;
    }

    async doRender$dom(ctx: TypstCancellationToken) {
      const condOrExit = <T,>(needFrame: boolean, cb: () => Promise<T>) => {
        if (needFrame && !ctx.isCancelRequested() && cb) {
          return cb();
        }
      };
      const pages = this.retrieveDOMPages().map(page => {
        const { innerWidth, innerHeight } = window;
        const browserBBox = page.getBoundingClientRect();
        // any part of the page is in the window
        return {
          inWindow: !(
            browserBBox.left > innerWidth ||
            browserBBox.right < 0 ||
            browserBBox.top > innerHeight ||
            browserBBox.bottom < 0
          ),
          page,
        };
      });
      const renderPage = async (i: number) => {
        await animationFrame();
        if (ctx.isCancelRequested()) {
          // console.log('cancel stage', RepaintStage.Layout, i);
          return undefined;
        }
        const page = pages[i].page;
        const browserBBox = page.getBoundingClientRect();
        const v = this.getDomViewport(window, browserBBox);

        const needCalc = (stage: RepaintStage) =>
          this.docKernel.need_repaint(i, v.x, v.y, v.width, v.height, stage);
        const repaint = (stage: RepaintStage) =>
          this.docKernel.repaint(i, v.x, v.y, v.width, v.height, stage);
        const calc = (stage: RepaintStage) => {
          if (ctx.isCancelRequested()) {
            return undefined;
          }
          return condOrExit(needCalc(stage), () => repaint(stage));
        };

        await calc(RepaintStage.Layout);

        const wScale =
          (browserBBox.width
            ? Number.parseFloat(page.getAttribute('data-width')!) / browserBBox.width
            : 1) * this.domScale;
        const hScale =
          (browserBBox.height
            ? Number.parseFloat(page.getAttribute('data-height')!) / browserBBox.height
            : 1) * this.domScale;
        v.x *= wScale;
        v.y *= hScale;
        v.y -= 100;
        v.width *= wScale;
        v.height *= hScale;
        v.height += 200;

        await calc(RepaintStage.Svg);
        await calc(RepaintStage.Semantics);
        if (ctx.isCancelRequested()) {
          // console.log('cancel stage', RepaintStage.Semantics, i);
          return undefined;
        }
        if (needCalc(RepaintStage.PrepareCanvas)) {
          const calcCanvasAfterPreparing = async () => {
            await repaint(RepaintStage.PrepareCanvas);
            if (ctx.isCancelRequested()) {
              return undefined;
            }
            return calc(RepaintStage.Canvas);
          };
          calcCanvasAfterPreparing();
        } else {
          await calc(RepaintStage.Canvas);
        }
      };
      const renderPages = async (inWindow: boolean) => {
        for (let idx = 0; idx < pages.length; ++idx) {
          if (ctx.isCancelRequested()) {
            // console.log('cancel page', RepaintStage.Layout, idx);
            return;
          }
          if (pages[idx].inWindow === inWindow) {
            await renderPage(idx);
          }
        }
      };

      this.cancelAnyway$dom();

      await renderPages(true);
      await renderPages(false);
      if (ctx.isCancelRequested()) {
        return;
      }
      // console.log('finished', RepaintStage.Layout);
    }
  };
}

export class TypstDomDocument extends provideDoc(
  composeDoc(
    TypstDocumentContext as GConstructor<TypstDocumentContext<InitDomDocArgs>>,
    provideDomDoc,
  ),
) { }
