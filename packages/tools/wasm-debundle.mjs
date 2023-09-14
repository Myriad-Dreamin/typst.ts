
import { readFileSync, writeFileSync, readdirSync, existsSync } from 'fs';

const pkgStats = readdirSync('pkg').map((fileName) => {
    if (!fileName.endsWith('.js')) {
        return;
    }
    
    const rustWasmFN = fileName.replace(/\.js$/, '_bg.wasm');

    if (!existsSync(`pkg/${rustWasmFN}`)) {
        return;
    }

    let bundleJs = readFileSync(`pkg/${fileName}`, 'utf8');

    // input = new URL('typst_ts_renderer_bg.wasm', import.meta.url)

    let replaced = [];
    const reg = /input = new URL\('(.+?)', import\.meta\.url\)/mg;
    let exp;
    while ((exp = reg.exec(bundleJs))) {
        console.log(`Found wasm file name: ${exp[1]}`);
        if (exp[1] === rustWasmFN) {
            console.log(exp.index, exp[0].length);
            // throw exception if not found
            replaced.push([exp.index, exp[0].length, rustWasmFN]);
        }
    }

    if (replaced.length === 0) {
        if (bundleJs.indexOf('wasm-debundle.mjs') >= 0) {
            return;
        }
        console.log(`No wasm file name found in ${fileName}`);
        return;
    }

    console.log(replaced);

    for (let i = replaced.length - 1; i >= 0; i--) {
        const [index, length, wasmFN] = replaced[i];
        bundleJs = bundleJs.substring(0, index) + `input = this.importWasmModule(${wasmFN}, import.meta.url)` + bundleJs.substring(index + length);
    }

    bundleJs = `/// Processed by wasm-debundle.mjs
` + bundleJs;

    console.log(`Processed ${rustWasmFN}...`);

    writeFileSync(`pkg/${fileName}`, bundleJs);
});
