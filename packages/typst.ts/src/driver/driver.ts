// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_renderer_ts_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_renderer_ts';

import type * as pdfjsModule from 'pdfjs-dist';
import type { TypstRendererInitOptions, BeforeBuildMark } from './options.init';

export interface RenderOptions {
  artifactContent: string;
  container: HTMLDivElement;
}

export interface TypstRenderer {
  init(options?: Partial<TypstRendererInitOptions>): Promise<void>;
  render(options: RenderOptions): Promise<RenderResult>;
}

export interface RenderResult {
  width: number;
  height: number;
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

  async init(options?: Partial<TypstRendererInitOptions>): Promise<void> {
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

  async renderImage(artifactContent: string): Promise<ImageData> {
    return this.renderer.render(artifactContent);
  }

  async renderPdf(artifactContent: string): Promise<Uint8Array> {
    return this.renderer.render_to_pdf(artifactContent);
  }

  private async renderDisplayLayer(
    artifactContent: string,
    container: HTMLDivElement,
  ): Promise<ImageData> {
    const containerWidth = container.offsetWidth;
    const imageScaleFactor = 2;

    const t = performance.now();

    const artifact = JSON.parse(artifactContent);

    const canvasList = new Array(artifact.pages.length);
    for (let i = 0; i < artifact.pages.length; i++) {
      const canvas = (canvasList[i] = document.createElement('canvas'));
      const ctx = canvas.getContext('2d');
      if (ctx) {
        const pageAst = artifact.pages[i];
        const width = Math.ceil(pageAst.size.x) * imageScaleFactor;
        const height = Math.ceil(pageAst.size.y) * imageScaleFactor;

        canvas.width = width;
        canvas.height = height;
      }

      container.appendChild(canvas);
    }

    const t2 = performance.now();

    const renderResult = await this.renderImage(artifactContent);
    let ctx = canvasList[0].getContext('2d');
    if (ctx) {
      ctx.putImageData(renderResult, 0, 0);
    }

    const t3 = performance.now();

    console.log(
      `display layer used: retieve/render = ${(t2 - t).toFixed(1)}/${(t3 - t2).toFixed(1)}ms`,
    );

    // compute scaling factor according to the paper size
    const currentScale = containerWidth / renderResult.width;
    container.style.transformOrigin = '0px 0px';
    container.style.transform = `scale(${currentScale})`;

    return renderResult;
  }

  private async renderOnePageTextLayer(
    container: HTMLElement,
    page: pdfjsModule.PDFPageProxy,
    scale: number,
  ) {
    const textContentSource = await page.getTextContent();
    this.pdf.renderTextLayer({
      textContentSource,
      container,
      viewport: page.getViewport({ scale }),
    });
  }

  private async renderTextLayer(artifact_content: string, container: HTMLDivElement) {
    const layer = document.getElementById('text-layer')!;
    const containerWidth = container.offsetWidth;
    const t2 = performance.now();

    const buf = await this.renderPdf(artifact_content);
    const doc = await this.pdf.getDocument(buf).promise;
    const t3 = performance.now();

    const page = await doc.getPage(1);

    // compute scale size
    const orignalScale = containerWidth / page.getViewport({ scale: 1 }).width;
    // the --scale-factor will truncate our scale, we do it first
    const scale = Number.parseFloat(orignalScale.toFixed(4));
    layer.parentElement?.style.setProperty('--scale-factor', scale.toString());

    this.renderOnePageTextLayer(layer, page, scale);
    const t4 = performance.now();

    console.log(
      `text layer used: retieve/render = ${(t3 - t2).toFixed(1)}/${(t4 - t3).toFixed(1)}ms`,
    );
  }

  async render({ artifactContent, container }: RenderOptions): Promise<RenderResult> {
    let renderResult: RenderResult;

    const doRenderDisplayLayer = async () => {
      renderResult = await this.renderDisplayLayer(artifactContent, container);
    };

    const doRenderTextLayer = new Promise(resolve => {
      setTimeout(() => {
        // setImmediate
        this.renderTextLayer(artifactContent, container).then(resolve);
      }, 0);
    });

    return Promise.all([doRenderDisplayLayer(), doRenderTextLayer]).then(() => {
      return {
        width: renderResult.width,
        height: renderResult.height,
      };
    });
  }
}

export function createTypstRenderer(pdf: typeof pdfjsModule): TypstRenderer {
  return new TypstRendererDriver(pdf);
}
