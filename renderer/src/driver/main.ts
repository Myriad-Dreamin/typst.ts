// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_renderer_ts_bg.wasm';
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_renderer_ts';

import * as pdfjsModule from 'pdfjs-dist';

export interface TypstRenderer {
  init(): Promise<void>;
  render(artifact_content: string, container: HTMLDivElement): Promise<RenderResult>;
}

export interface RenderResult {
  width: number;
  height: number;
}

class TypstRendererImpl {
  renderer: typst.TypstRenderer;

  constructor(private pdf: typeof pdfjsModule) {}

  async loadFont(builder: typst.TypstRendererBuilder, font_path: string): Promise<void> {
    const response = await fetch(font_path);
    const font_buffer = await response.arrayBuffer();
    await builder.add_raw_font(new Uint8Array(font_buffer));
  }

  async init(): Promise<void> {
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

  async renderImage(artifact_content: string): Promise<ImageData> {
    return this.renderer.render(artifact_content);
  }

  async renderPdf(artifact_content: string): Promise<Uint8Array> {
    return this.renderer.render_to_pdf(artifact_content);
  }

  private async renderDisplayLayer(
    artifact_content: string,
    imageContainer: HTMLDivElement,
  ): Promise<ImageData> {
    let canvas = document.createElement('canvas');

    const imageContainerWidth = imageContainer.offsetWidth;

    const t = performance.now();
    const imageRenderResult = await this.renderImage(artifact_content);
    const t2 = performance.now();

    canvas.width = imageRenderResult.width;
    canvas.height = imageRenderResult.height;
    let ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.putImageData(imageRenderResult, 0, 0);
    }

    const t3 = performance.now();
    console.log('time used', t2 - t, t3 - t2);

    // compute scaling factor according to the paper size
    const currentScale = imageContainerWidth / imageRenderResult.width;
    imageContainer.style.transformOrigin = '0px 0px';
    imageContainer.style.transform = `scale(${currentScale})`;

    imageContainer.appendChild(canvas);

    return imageRenderResult;
  }

  private async renderTextLayer(artifact_content: string, imageContainer: HTMLDivElement) {
    const imageContainerWidth = imageContainer.offsetWidth;

    const t2 = performance.now();
    const layer = document.getElementById('text-layer');
    const data = await this.renderPdf(artifact_content);
    const pdfDoc = await this.pdf.getDocument(data).promise;
    const t3 = performance.now();

    const page = await pdfDoc.getPage(1);
    const textLayerScale = Number.parseFloat(
      (imageContainerWidth / page.getViewport({ scale: 1 }).width).toFixed(4),
    );
    layer?.parentElement?.style.setProperty('--scale-factor', textLayerScale.toString());

    page.getTextContent().then(textContent => {
      console.log(textContent);

      this.pdf.renderTextLayer({
        textContentSource: textContent,
        container: layer!,
        viewport: page.getViewport({ scale: textLayerScale }),
      });
      const t4 = performance.now();

      console.log('text layer used', t3 - t2, t4 - t3);
    });
  }

  async render(artifact_content: string, imageContainer: HTMLDivElement): Promise<RenderResult> {
    let renderResult: RenderResult;

    const doRenderDisplayLayer = async () => {
      renderResult = await this.renderDisplayLayer(artifact_content, imageContainer);
    };

    const doRenderTextLayer = new Promise(resolve => {
      setTimeout(() => {
        // setImmediate
        this.renderTextLayer(artifact_content, imageContainer).then(resolve);
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
  return new TypstRendererImpl(pdf);
}

// Export module on window.
// todo: graceful way?
if (window) {
  (window as any).createTypstRenderer = createTypstRenderer;
}
