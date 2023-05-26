import { PageInfo } from './internal.types';
import { RenderOptionsBase } from './options.render';
import { PageViewport } from './viewport';
import type * as pdfjsModule from 'pdfjs-dist';

/** @internal */
export class RenderView {
  loadPageCount: number;
  imageScaleFactor: number;
  partialPageRendering: boolean;

  container: HTMLDivElement;
  canvasList: HTMLCanvasElement[];
  layerList: HTMLDivElement[];
  commonList: HTMLDivElement[];
  textLayerParentList: HTMLDivElement[];

  constructor(public pageInfos: PageInfo[], container: HTMLDivElement, options: RenderOptionsBase) {
    this.partialPageRendering = options.pages !== undefined;
    this.imageScaleFactor = options.pixelPerPt ?? 2;

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

  resetLayout() {
    /// resize again to avoid bad width change after render
    if (this.partialPageRendering) {
      for (let i = 0; i < this.pageInfos.length; i++) {
        const pageAst = this.pageInfos[i];
        const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
        const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

        const canvasDiv = this.canvasList[i].parentElement;
        if (!canvasDiv) {
          throw new Error(
            `canvasDiv is null for page ${i}, canvas list length ${this.canvasList.length}`,
          );
        }
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

        const canvasDiv = this.canvasList[i].parentElement;
        if (!canvasDiv) {
          throw new Error(
            `canvasDiv is null for page ${i}, canvas list length ${this.canvasList.length}`,
          );
        }
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

export function renderTextLayer(
  pdfjsLib: unknown,
  container: HTMLDivElement,
  pageInfos: PageInfo[],
  layerList: HTMLDivElement[],
  textSourceList: any[],
) {
  const containerWidth = container.offsetWidth;
  const t2 = performance.now();

  const renderOne = (layer: HTMLDivElement, i: number) => {
    const page_info = pageInfos[i];
    if (!page_info) {
      console.error('page not found for', i);
      return;
    }
    const width_pt = page_info.width;
    const height_pt = page_info.height;
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

    (pdfjsLib as typeof pdfjsModule).renderTextLayer({
      textContentSource: textSourceList[i],
      container: layer,
      viewport,
    });
  };

  layerList.forEach(renderOne);
  const t3 = performance.now();
  console.log(`text layer used: render = ${(t3 - t2).toFixed(1)}ms`);
}
