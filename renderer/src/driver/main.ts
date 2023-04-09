
// @ts-ignore
import typst_wasm_bin from '../../pkg/typst_renderer_ts_bg.wasm'
// @ts-ignore
import typstInit, * as typst from '../../pkg/typst_renderer_ts'


export interface TypstRenderer {
    init(): Promise<void>;
    render(artifact_content: string): Promise<ImageData>;
}


class TypstRendererImpl {
    renderer: typst.TypstRenderer;

    async init(): Promise<void> {
        await typstInit(typst_wasm_bin)
        this.renderer = await new typst.TypstRenderer();
        console.log("loaded Typst");
    }

    async render(artifact_content: string): Promise<ImageData> {
        const t = performance.now();
        const renderResult = this.renderer.render(artifact_content);
        console.log(renderResult);
        const t2 = performance.now();
        console.log("time used", t2-t);
        return renderResult;
    }
}

export function createTypstRenderer(): TypstRenderer {
    return new TypstRendererImpl();
}

// Export module on window.
// todo: graceful way?
if (window) {
    (window as any).createTypstRenderer = createTypstRenderer;
}
