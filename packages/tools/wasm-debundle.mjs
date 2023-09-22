
import { readFileSync, writeFileSync, readdirSync, existsSync, unlinkSync } from 'fs';

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
        bundleJs = bundleJs.substring(0, index) + `input = importWasmModule('${wasmFN}', import.meta.url)` + bundleJs.substring(index + length);
    }

    bundleJs = `/// Processed by wasm-debundle.mjs
` + bundleJs + `

let importWasmModule = async function(wasm_name, url) {
    throw new Error('Cannot import wasm module without importer: ' + wasm_name + ' ' + url);
};
function setImportWasmModule(importer) {
  importWasmModule = importer;
}
export {
  setImportWasmModule
}
`;

    console.log(`Processed ${rustWasmFN}...`);

    unlinkSync(`pkg/${fileName}`);
    // rewrite extension: .js -> .mjs
    fileName = fileName.replace(/\.js$/, '.mjs');
    writeFileSync(`pkg/${fileName}`, bundleJs);
    writeFileSync(`pkg/wasm-pack-shim.mjs`, `
import { setImportWasmModule } from './${fileName}';
import _default from './${fileName}';
export * from './${fileName}';
export default _default;

let nodeJsImportWasmModule = async function(wasm_name, url) {
  const escape_import = t => import(t);
  const path = await escape_import('path');
  const { readFileSync } = await escape_import('fs');

  const wasmPath = new URL(path.join(path.dirname(url), wasm_name));
  return await readFileSync(wasmPath).buffer;
};

// nodejs
const isNode =
  typeof process !== "undefined" &&
  process.versions != null &&
  process.versions.node != null;

if (isNode) {
  setImportWasmModule(nodeJsImportWasmModule);
}

`);
writeFileSync(`pkg/wasm-pack-shim.d.mts`, `
import _default from './${fileName}';
export * from './${fileName}';
export default _default;
`);

});
