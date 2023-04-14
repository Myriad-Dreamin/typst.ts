
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

    async loadFont(builder: typst.TypstRendererBuilder, font_path: string): Promise<void> {
        const response = await fetch(font_path);
        const font_buffer = await response.arrayBuffer();
        await builder.add_raw_font(new Uint8Array(font_buffer));
    }

    async init(): Promise<void> {
        await typstInit(typst_wasm_bin)
        let builder = new typst.TypstRendererBuilder();

        const t = performance.now();
        await Promise.all([
            this.loadFont(builder, "dist/fonts/LinLibertine_R.ttf"),
            this.loadFont(builder, "dist/fonts/LinLibertine_RB.ttf"),
            this.loadFont(builder, "dist/fonts/LinLibertine_RBI.ttf"),
            this.loadFont(builder, "dist/fonts/LinLibertine_RI.ttf"),
            this.loadFont(builder, "dist/fonts/NewCMMath-Book.otf"),
            this.loadFont(builder, "dist/fonts/NewCMMath-Regular.otf"),
        ])

        if ('queryLocalFonts' in window) {
            const fonts = await (window as any).queryLocalFonts();
            for (const font of fonts) {
                if (!font.family.includes('Segoe UI Symbol')) {
                    continue;
                }
                const data: ArrayBuffer = await (await font.blob()).arrayBuffer();
                await builder.add_raw_font(new Uint8Array(data));
            }
        }
        const t2 = performance.now();
        console.log("fond loading", t2-t);

        // todo: search browser
        // searcher.search_browser().await?;

        this.renderer = await builder.build();
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
