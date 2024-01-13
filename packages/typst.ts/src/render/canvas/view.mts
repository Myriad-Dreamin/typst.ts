import { PageInfo, TypstDefaultParams } from '../../internal.types.mjs';
import { RenderToCanvasOptions } from '../../options.render.mjs';
import { PageViewport } from './viewport.mjs';

/** @internal */
export class RenderView {
  loadPageCount: number;
  imageScaleFactor: number;

  container: HTMLElement;
  canvasList: HTMLCanvasElement[];
  textLayerList: HTMLDivElement[];
  annotationLayerList: HTMLDivElement[];
  commonList: HTMLDivElement[];
  textLayerParentList: HTMLDivElement[];
  semanticLayerList: HTMLDivElement[];

  constructor(
    public pageInfos: PageInfo[],
    container: HTMLElement,
    options: RenderToCanvasOptions,
  ) {
    this.imageScaleFactor = options.pixelPerPt ?? TypstDefaultParams.PIXEL_PER_PT;

    container.innerHTML = '';
    container.style.width = '100%';

    // canvas[data-typst-session='{}']

    /// refer html elements
    this.container = container;
    this.canvasList = new Array(this.loadPageCount);
    this.textLayerList = new Array(this.loadPageCount);
    this.commonList = new Array(this.loadPageCount);
    this.textLayerParentList = new Array(this.loadPageCount);
    this.annotationLayerList = new Array(this.loadPageCount);
    this.semanticLayerList = new Array(this.loadPageCount);

    const createOver = (i: number, width: number, height: number, commonDiv: HTMLDivElement) => {
      const canvas = (this.canvasList[i] = document.createElement('canvas'));
      const semanticLayer = (this.semanticLayerList[i] = document.createElement('div'));
      const textLayer = (this.textLayerList[i] = document.createElement('div'));
      const textLayerParent = (this.textLayerParentList[i] = document.createElement('div'));
      const annotationLayer = (this.annotationLayerList[i] = document.createElement('div'));

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
        textLayerParent.style.position = 'absolute';
        annotationLayer.style.width = `${containerWidth}px`;
        annotationLayer.style.height = `${height * orignalScale}px`;
        annotationLayer.style.position = 'absolute';
        commonDiv.classList.add('typst-page');
        commonDiv.classList.add('canvas');
        commonDiv.style.width = `${containerWidth}px`;
        commonDiv.style.height = `${height * orignalScale}px`;
        commonDiv.style.position = 'relative';

        // textLayerParent.style.zIndex = '1';
        semanticLayer.appendChild(textLayerParent);
        semanticLayer.appendChild(annotationLayer);
        commonDiv.appendChild(semanticLayer);
      }
    };

    for (let i = 0; i < this.pageInfos.length; i++) {
      const pageAst = this.pageInfos[i];
      const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
      const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

      // const commonDiv = document.createElement('div');
      let commonDiv: HTMLDivElement | undefined = undefined;

      commonDiv = this.commonList[i] = document.createElement('div');
      container.appendChild(commonDiv);
      createOver(i, width, height, commonDiv);
    }
  }

  resetLayout() {
    /// resize again to avoid bad width change after render
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
      const annotationLayer = this.annotationLayerList[i];

      /// on width change
      const containerWidth = this.container.offsetWidth;
      const orignalScale = containerWidth / width;
      textLayerParent.style.width = `${containerWidth}px`;
      textLayerParent.style.height = `${height * orignalScale}px`;
      annotationLayer.style.width = `${containerWidth}px`;
      annotationLayer.style.height = `${height * orignalScale}px`;
      commonDiv.style.width = `${containerWidth}px`;
      commonDiv.style.height = `${height * orignalScale}px`;

      // compute scaling factor according to the paper size
      const currentScale = this.container.offsetWidth / width;
      canvasDiv.style.transformOrigin = '0px 0px';
      canvasDiv.style.transform = `scale(${currentScale})`;
    }
  }
}

export function renderTextLayer(
  pdfjsLib: any,
  container: HTMLElement,
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

    pdfjsLib.renderTextLayer({
      textContentSource: textSourceList[i],
      container: layer,
      viewport,
    });
  };

  layerList.forEach(renderOne);
  const t3 = performance.now();
  console.log(`text layer used: render = ${(t3 - t2).toFixed(1)}ms`);
}
