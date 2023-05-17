// @ts-ignore
import * as typst from '../../pkg/typst_ts_renderer';

import { PageInfo, RenderSession } from './internal.types';
import { RenderOptionsBase } from './options.render';

/** @internal */
export class RenderView {
  loadPageCount: number;
  imageScaleFactor: number;
  partialPageRendering: boolean;
  pageInfos: PageInfo[];

  container: HTMLDivElement;
  canvasList: HTMLCanvasElement[];
  layerList: HTMLDivElement[];
  commonList: HTMLDivElement[];
  textLayerParentList: HTMLDivElement[];

  constructor(session: typst.RenderSession, container: HTMLDivElement, options: RenderOptionsBase) {
    this.partialPageRendering = options.pages !== undefined;
    this.pageInfos = [];
    this.loadPageCount = options.pages ? options.pages.length : session.pages_info.page_count;
    this.imageScaleFactor = options.pixelPerPt ?? 2;
    this.loadPagesInfo(session, options);

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

    /// refer html elements
    this.container = container;
    this.canvasList = new Array(this.loadPageCount);
    this.layerList = new Array(this.loadPageCount);
    this.commonList = new Array(this.loadPageCount);
    this.textLayerParentList = new Array(this.loadPageCount);

    const createOver = (i: number, width: number, height: number, commonDiv: HTMLDivElement) => {
      const canvas = (this.canvasList[i] = document.createElement('canvas'));
      const textLayer = (this.layerList[i] = document.createElement('div'));
      const textLayerParent = (this.textLayerParentList[i] = document.createElement('div'));

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
    };

    if (options.pages) {
      for (let i = 0; i < this.pageInfos.length; i++) {
        const pageAst = this.pageInfos[i];
        const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
        const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

        // const commonDiv = document.createElement('div');
        let commonDiv = undefined;

        while (pageAst.pageOffset >= commonDivList.length) {
          const elem = document.createElement('div');
          commonDivList.push(elem);
          container.appendChild(elem);
        }
        commonDiv = this.commonList[i] = commonDivList[pageAst.pageOffset];
        if (commonDiv) {
          commonDiv.innerHTML = '';
        }

        createOver(i, width, height, commonDiv);
      }
    } else {
      for (let i = 0; i < this.pageInfos.length; i++) {
        const pageAst = this.pageInfos[i];
        const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
        const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

        // const commonDiv = document.createElement('div');
        let commonDiv = undefined;

        commonDiv = this.commonList[i] = document.createElement('div');
        container.appendChild(commonDiv);
        createOver(i, width, height, commonDiv);
      }
    }
  }

  loadPagesInfo(session: typst.RenderSession, options: RenderOptionsBase): void {
    const pages_info = session.pages_info;
    if (options.pages) {
      for (let i = 0; i < this.loadPageCount; i++) {
        const pageNum = options.pages[i].number;
        const pageAst = pages_info.page_by_number(pageNum);
        if (!pageAst) {
          throw new Error(`page ${pageNum} is not loaded`);
        }
        this.pageInfos.push({
          pageOffset: pageAst.page_off,
          width: pageAst.width_pt,
          height: pageAst.height_pt,
        });
      }
    } else {
      for (let i = 0; i < this.loadPageCount; i++) {
        const pageAst = pages_info.page(i);
        this.pageInfos.push({
          pageOffset: pageAst.page_off,
          width: pageAst.width_pt,
          height: pageAst.height_pt,
        });
      }
    }
  }

  resetLayout() {
    /// resize again to avoid bad width change after render
    if (this.partialPageRendering) {
      for (let i = 0; i < this.pageInfos.length; i++) {
        const pageAst = this.pageInfos[i];
        const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
        const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

        const canvasDiv = this.canvasList[i].parentElement!;
        const commonDiv = this.commonList[i];
        const textLayerParent = this.textLayerParentList[i];

        /// on width change
        const containerWidth = this.container.offsetWidth;
        const orignalScale = containerWidth / width;
        textLayerParent.style.width = `${containerWidth}px`;
        textLayerParent.style.height = `${height * orignalScale}px`;
        commonDiv.style.width = `${containerWidth}px`;
        commonDiv.style.height = `${height * orignalScale}px`;

        // compute scaling factor according to the paper size
        const currentScale = this.container.offsetWidth / width;
        canvasDiv.style.transformOrigin = '0px 0px';
        canvasDiv.style.transform = `scale(${currentScale})`;
      }
    } else {
      for (let i = 0; i < this.pageInfos.length; i++) {
        const pageAst = this.pageInfos[i];
        const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
        const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

        const canvasDiv = this.canvasList[i].parentElement!;
        const commonDiv = this.commonList[i];
        const textLayerParent = this.textLayerParentList[i];

        /// on width change
        const containerWidth = this.container.offsetWidth;
        const orignalScale = containerWidth / width;
        textLayerParent.style.width = `${containerWidth}px`;
        textLayerParent.style.height = `${height * orignalScale}px`;
        commonDiv.style.width = `${containerWidth}px`;
        commonDiv.style.height = `${height * orignalScale}px`;

        // compute scaling factor according to the paper size
        const currentScale = this.container.offsetWidth / width;
        canvasDiv.style.transformOrigin = '0px 0px';
        canvasDiv.style.transform = `scale(${currentScale})`;
      }
    }
  }
}
