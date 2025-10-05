/// <reference types="@vitest/browser/context" />

import { describe, expect, it } from 'vitest';
// todo: why does it give errors?
import rendererUrl from '../../renderer/pkg/typst_ts_renderer_bg.wasm?url';
import { page, commands } from '@vitest/browser/context';

import { createTypstRenderer } from '../src/renderer.mjs';

// nodejs
const isNode =
    typeof process !== 'undefined' && process.versions != null && process.versions.node != null;

const fsImport = (file: string) => {
    const fs = require('fs');
    const path = require('path');
    return fs.readFileSync(path.join(import.meta.dirname, file));
};

const getModule = () => {
    const compiler = () => {
        throw new Error("shouldn't load compiler when testing renderer")
    };
    if (isNode) {
        return {
            compiler,
            renderer: () => fsImport('../../../../renderer/pkg/typst_ts_renderer_bg.wasm'),
        };
    }
    return {
        compiler,
        renderer: () => rendererUrl,
    };
};

const getRenderer = async () => {
    const r = createTypstRenderer();
    await r.init({ getModule: getModule().renderer });
    return r;
};

export const testSvg = async (data: Uint8Array) => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const rendered = await renderer.runWithSession(async renderSession => {
        renderer.manipulateData({
            renderSession,
            action: 'reset',
            data,
        });
        return await renderer.renderSvg({
            renderSession,
        });
    });
    container.innerHTML = rendered;
    const width = Number.parseFloat((container.firstElementChild as any).dataset.width);
    const height = Number.parseFloat((container.firstElementChild as any).dataset.height);
    page.viewport(width, height);
    document.body.appendChild(container);
    return { container, width, height };
};

export const testCanvas = async (data: Uint8Array) => {
    const container = document.createElement('div');
    const renderer = await getRenderer();
    const { width, height } = await renderer.runWithSession(async renderSession => {
        renderer.manipulateData({
            renderSession,
            action: 'reset',
            data,
        });
        const width = renderSession.docWidth;
        const height = renderSession.docHeight;
        page.viewport(width, height);
        await renderer.renderToCanvas({
            renderSession,
            container,
        });

        return { width, height };
    });

    return { container, width, height };
};

export const makeSnapshot = async (s: { container: HTMLElement, width: number, height: number }, name: string) => {
    const snapshotPath = await page.screenshot({ save: true, path: `../../screenshots/renderer/${name}` });
    const { createSnapshot } = commands as any;
    const ret = await createSnapshot(snapshotPath, name, `size=${s.width}x${s.height}`);
    // screenshotHash, refHash
    expect(ret.screenshotHash).toEqual(ret.refHash);
};
