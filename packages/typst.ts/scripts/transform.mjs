// mkdirSync('./dist/cjs', { recursive: true });

// walk the directory tree `./dist/cjs`

import { readdirSync, statSync, readFileSync, writeFileSync, renameSync } from 'fs';
import * as esbuild from 'esbuild';
import { execSync } from 'child_process';

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

const runTransform = (inp, isDcts) => {
  // replace Promise.resolve().then(() => require('@myriaddreamin/typst-ts-web-compiler/pkg/wasm-pack-shim.mjs'))
  // to import('@myriaddreamin/typst-ts-web-compiler/pkg/wasm-pack-shim.mjs')

  // only replace (require("*.mjs")) to (require("*.cjs"))
  // and  replace (from "*.mjs") to (from "*.cjs")
  const mid = inp
    .replace(
      /Promise\s*\.\s*resolve\s*\(\s*\)\s*\.\s*then\s*\(\s*\(\s*\)\s*=>\s*require\(['"](.*?)['"]\)\s*\)/g,
      'import("$1")',
    )
    .replace(/import\(\s*['"].\/(.*?)\.mjs['"]\s*\)/g, 'import(".\/$1.cjs")')
    .replace(/require\(\s*['"](.*?)\.mjs['"]\s*\)/g, 'require("$1.cjs")')
    .replace(/from\s*['"](.*?)\.mjs['"]/g, 'from "$1.cjs"');
  if (isDcts) {
    return mid;
  }

  const res = esbuild.transformSync(mid, {
    loader: 'js',
    format: 'cjs',
  });
  return res.code;
};

const generatedExports = {
  '.': {
    require: {
      types: './dist/cjs/index.d.cts',
      default: './dist/cjs/index.cjs',
    },
    import: {
      types: './dist/esm/index.d.ts',
      default: './dist/esm/index.mjs',
    },
  },
  './renderer': {
    require: {
      types: './dist/cjs/renderer.d.cts',
      default: './dist/cjs/renderer.cjs',
    },
    import: {
      types: './dist/esm/renderer.d.mts',
      default: './dist/esm/renderer.mjs',
    },
  },
  './contrib/global-renderer': {
    require: {
      types: './dist/cjs/contrib/global-renderer.d.cts',
      default: './dist/cjs/contrib/global-renderer.cjs',
    },
    import: {
      types: './dist/esm/contrib/global-renderer.d.mts',
      default: './dist/esm/contrib/global-renderer.mjs',
    },
  },
  './compiler': {
    require: {
      types: './dist/cjs/compiler.d.cts',
      default: './dist/cjs/compiler.cjs',
    },
    import: {
      types: './dist/esm/compiler.d.mts',
      default: './dist/esm/compiler.mjs',
    },
  },
  './contrib/global-compiler': {
    require: {
      types: './dist/cjs/contrib/global-compiler.d.cts',
      default: './dist/cjs/contrib/global-compiler.cjs',
    },
    import: {
      types: './dist/esm/contrib/global-compiler.d.mts',
      default: './dist/esm/contrib/global-compiler.mjs',
    },
  },
  './*': ['./*', './*.d.mts'],
};

walk('./dist/cjs', path => {
  const newPath = path.replace(/\.mjs$/g, '.cjs').replace(/\.mts$/g, '.cts');
  if (newPath !== path) {
    renameSync(path, newPath);
  }
  const isCjs = newPath.endsWith('.cjs');
  const isCts = newPath.endsWith('.cts');
  if (isCjs || isCts) {
    writeFileSync(newPath, runTransform(readFileSync(newPath, { encoding: 'utf-8' }), isCts));
  }

  if (isCjs) {
    const esmPath = newPath
      .replace('dist/cjs', 'dist/esm')
      .replace(/\.cjs$/g, '.mjs')
      .replace(/\.cts$/g, '.mts');
    const desmPath = esmPath.replace(/\.mjs$/g, '.d.mts');
    const dctsPath = newPath.replace(/\.cjs$/g, '.d.cts');
    const commonPath = newPath
      .replace('./dist/cjs/', '')
      .replace('./dist/cjs/', '')
      .replace(/\.cjs$/g, '');

    generatedExports[`./${commonPath}`] = {
      require: {
        types: dctsPath,
        default: newPath,
      },
      import: {
        types: desmPath,
        default: esmPath,
      },
    };
  }
});

const pkgJson = JSON.parse(readFileSync('./package.json'));
pkgJson.exports = generatedExports;
writeFileSync('./package.json', JSON.stringify(pkgJson, null, 2) + '\n');
