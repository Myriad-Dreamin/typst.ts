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
    this.semanticLayerList = new Array(this.loadPageCount);

    const createOver = (i: number, pageAst: PageInfo, commonDiv: HTMLDivElement) => {
      const width = Math.ceil(pageAst.width) * this.imageScaleFactor;
      const height = Math.ceil(pageAst.height) * this.imageScaleFactor;

      const canvas = (this.canvasList[i] = document.createElement('canvas'));
      const semanticLayer = (this.semanticLayerList[i] = document.createElement('div'));
      const textLayer = (this.textLayerList[i] = document.createElement('div'));
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

        textLayerParent.className = 'typst-html-semantics';

        /// on width change
        const containerWidth = container.offsetWidth;
        const originalScale = containerWidth / pageAst.width;
        textLayerParent.style.width = `${containerWidth}px`;
        textLayerParent.style.height = `${pageAst.height * originalScale}px`;
        // --data-text-width
        textLayerParent.style.setProperty('--data-text-width', `${originalScale}px`);
        textLayerParent.style.setProperty('--data-text-height', `${originalScale}px`);
        // textLayerParent.style.position = 'absolute';
        commonDiv.classList.add('typst-page');
        commonDiv.classList.add('canvas');
        commonDiv.style.width = `${containerWidth}px`;
        commonDiv.style.height = `${height * originalScale}px`;
        commonDiv.style.position = 'relative';

        // textLayerParent.style.zIndex = '1';
        semanticLayer.appendChild(textLayerParent);
        commonDiv.appendChild(semanticLayer);
      }
    };

    for (let i = 0; i < this.pageInfos.length; i++) {
      const pageAst = this.pageInfos[i];

      // const commonDiv = document.createElement('div');
      let commonDiv: HTMLDivElement | undefined = undefined;

      commonDiv = this.commonList[i] = document.createElement('div');
      container.appendChild(commonDiv);
      createOver(i, pageAst, commonDiv);
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

      /// on width change
      const containerWidth = this.container.offsetWidth;
      const originalScale = containerWidth / width;
      textLayerParent.style.width = `${containerWidth}px`;
      textLayerParent.style.height = `${height * originalScale}px`;
      commonDiv.style.width = `${containerWidth}px`;
      commonDiv.style.height = `${height * originalScale}px`;

      // compute scaling factor according to the paper size
      const currentScale = this.container.offsetWidth / width;
      canvasDiv.style.transformOrigin = '0px 0px';
      canvasDiv.style.transform = `scale(${currentScale})`;
    }
  }
}
