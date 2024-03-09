
// mkdirSync('./dist/cjs', { recursive: true });

// walk the directory tree `./dist/cjs`

import { readdirSync, statSync, readFileSync, writeFileSync, renameSync } from 'fs';

function walk(path, cb) {
  const files = readdirSync(path);
    for (const file of files) {
        const curPath = path + '/' + file;
        if (statSync(curPath).isDirectory()) {
        // recurse
        walk(curPath, cb);
        } else {
            cb(curPath);
        }
    }
}


walk('./dist/cjs', (path) => {
    const newPath = path.replace(/\.mjs/g, '.cjs').replace(/\.mts/g, '.cts');
    renameSync(path, newPath);
    if (newPath !== path && (newPath.endsWith('.cjs') || newPath.endsWith('.cts'))) {
        /// rename content
        const content = readFileSync(newPath).toString();
        // replace Promise.resolve().then(() => require('@myriaddreamin/typst-ts-web-compiler/pkg/wasm-pack-shim.mjs'))
        // to import('@myriaddreamin/typst-ts-web-compiler/pkg/wasm-pack-shim.mjs')
        
        // only replace (require("*.mjs")) to (require("*.cjs"))
        // and  replace (from "*.mjs") to (from "*.cjs")
        const newContent = content
        .replace(/Promise\s*\.\s*resolve\s*\(\s*\)\s*\.\s*then\s*\(\s*\(\s*\)\s*=>\s*require\(['"](.*?)['"]\)\s*\)/g, 'import("$1")')
        .replace(/import\(\s*['"].\/(.*?)\.mjs['"]\s*\)/g, 'import(".\/$1.cjs")')
        .replace(/require\(\s*['"](.*?)\.mjs['"]\s*\)/g, 'require("$1.cjs")').replace(/from\s*['"](.*?)\.mjs['"]/g, 'from "$1.cjs"');
        writeFileSync(newPath, newContent);
    }
});

