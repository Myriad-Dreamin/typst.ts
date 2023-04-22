// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_renderer_ts_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_renderer_ts';

import type * as pdfjsModule from 'pdfjs-dist';

export interface TypstRendererInitOptions {
    
}

export interface TypstRenderer {
  init(options?: Partial<TypstRendererInitOptions>): Promise<void>;
  render(artifactContent: string, container: HTMLDivElement): Promise<RenderResult>;
}

export interface RenderResult {
  width: number;
  height: number;
}

class TypstRendererDriver {
  renderer: typst.TypstRenderer;

  constructor(private pdf: typeof pdfjsModule) {}

  async loadFont(builder: typst.TypstRendererBuilder, fontPath: string): Promise<void> {
    const response = await fetch(fontPath);
    const fontBuffer = new Uint8Array(await response.arrayBuffer());
    await builder.add_raw_font(fontBuffer);
  }

  async init(options?: Partial<TypstRendererInitOptions>): Promise<void> {
    await typstInit(typst_wasm_bin);
    let builder = new typst.TypstRendererBuilder();

    await Promise.all([
      this.loadFont(builder, 'dist/fonts/LinLibertine_R.ttf'),
      this.loadFont(builder, 'dist/fonts/LinLibertine_RB.ttf'),
      this.loadFont(builder, 'dist/fonts/LinLibertine_RBI.ttf'),
      this.loadFont(builder, 'dist/fonts/LinLibertine_RI.ttf'),
      this.loadFont(builder, 'dist/fonts/NewCMMath-Book.otf'),
      this.loadFont(builder, 'dist/fonts/NewCMMath-Regular.otf'),
    ]);

    const t = performance.now();
    if ('queryLocalFonts' in window) {
      const fonts = await (window as any).queryLocalFonts();
      console.log('local fonts count:', fonts.length);

      for (const font of fonts) {
        if (!font.family.includes('Segoe UI Symbol')) {
          continue;
        }

        const data: ArrayBuffer = await (await font.blob()).arrayBuffer();
        await builder.add_raw_font(new Uint8Array(data));
    }
    }

    const t2 = performance.now();
    console.log('font loading', t2 - t);

    // todo: search browser
    // searcher.search_browser().await?;

    this.renderer = await builder.build();
    console.log('loaded Typst');
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

  async render(artifactContent: string, container: HTMLDivElement): Promise<RenderResult> {
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
