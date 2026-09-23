// Test ① & ③ for proposal #5 (typst.ts#764):
// ① the publish directory of the built package contains the stylesheet and both
//    prebuilt bundles, and the `exports["."]` entries resolve to real files and
//    re-export `TypstDocument` without any `?inline` specifier;
// ③ `npm pack --dry-run` lists every file a consumer needs.
//
// Run via `pnpm test` in packages/typst.react (build runs first).

import assert from 'node:assert/strict';
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const pkgDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const pkg = JSON.parse(fs.readFileSync(path.join(pkgDir, 'package.json'), 'utf8'));

const REQUIRED_DIST_FILES = [
  'typst.css', // the stylesheet itself (#764: it was never emitted)
  'typst.react.mjs', // prebuilt ESM bundle, stylesheet already inlined
  'typst.react.js', // prebuilt CJS bundle, stylesheet already inlined
  'index.d.ts', // `exports["."].types`
];

test('publish directory contains stylesheet and both prebuilt bundles', () => {
  for (const f of REQUIRED_DIST_FILES) {
    assert.ok(
      fs.existsSync(path.join(pkgDir, 'dist', f)),
      `dist/${f} is missing; run \`pnpm build\` first`,
    );
  }
});

test('exports["."] points at the prebuilt bundles and the files exist', () => {
  const entry = pkg.exports['.'];
  assert.equal(entry.import, './dist/typst.react.mjs');
  assert.equal(entry.require, './dist/typst.react.js');
  assert.equal(entry.types, './dist/index.d.ts');
  for (const key of ['import', 'require', 'types']) {
    assert.ok(
      fs.existsSync(path.join(pkgDir, entry[key])),
      `exports["."].${key} (${entry[key]}) does not exist`,
    );
  }
});

test('the entry bundle re-exports TypstDocument and has no ?inline specifier', () => {
  const mjs = fs.readFileSync(path.join(pkgDir, 'dist', 'typst.react.mjs'), 'utf8');
  assert.ok(!mjs.includes('?inline'), 'entry bundle still contains a ?inline specifier');
  assert.match(mjs, /TypstDocument/, 'entry bundle does not mention TypstDocument');
  assert.ok(
    /export\s*\{[^}]*\bTypstDocument\b[^}]*\}/.test(mjs),
    'entry bundle does not export TypstDocument',
  );
  // The stylesheet is inlined into the bundles, so a consumer needs no css loader.
  const cssSource = fs.readFileSync(path.join(pkgDir, 'src', 'lib', 'typst.css'), 'utf8');
  const marker = cssSource.match(/[.#][a-zA-Z][\w-]*/)[0];
  assert.ok(
    mjs.includes(marker) ||
      fs.readFileSync(path.join(pkgDir, 'dist', 'typst.react.js'), 'utf8').includes(marker),
    `stylesheet marker ${marker} not found in either prebuilt bundle`,
  );
});

const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';

test('npm pack lists stylesheet, both bundles and the type entry', () => {
  // Through the shell: on Windows npm is a .cmd shim that plain spawn refuses.
  const out = execSync(`${npmCmd} pack --dry-run --json`, {
    cwd: pkgDir,
    encoding: 'utf8',
  });
  const files = JSON.parse(out)[0].files.map(f => f.path);
  for (const f of ['package.json', ...REQUIRED_DIST_FILES.map(f => `dist/${f}`)]) {
    assert.ok(files.includes(f), `npm pack listing is missing ${f}`);
  }
});
