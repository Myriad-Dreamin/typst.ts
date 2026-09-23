// Test ② for proposal #5 (typst.ts#764): a non-Vite consumer (webpack)
// reproduces the issue against the published 0.8.0-rc3 and goes green once the
// published package's `exports["."]` is repointed to what this branch ships.
//
// Run via `pnpm test` in packages/typst.react. Needs network once, to pack
// @myriaddreamin/typst.react@0.8.0-rc3 from the registry.

import assert from 'node:assert/strict';
import { execSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const pkgDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const branchExports = JSON.parse(fs.readFileSync(path.join(pkgDir, 'package.json'), 'utf8')).exports;

const npmCmd = process.platform === 'win32' ? 'npm.cmd' : 'npm';

function run(cmd, args, cwd) {
  // Quote every argument and go through the shell: on Windows, npm/npx are
  // .cmd shims that plain spawn refuses (EINVAL) and cmd.exe needs quoting.
  const cmdline = [cmd, ...args.map(a => `"${a}"`)].join(' ');
  try {
    return { code: 0, output: execSync(cmdline, { cwd, encoding: 'utf8' }) };
  } catch (e) {
    return { code: e.status ?? 1, output: `${e.stdout ?? ''}${e.stderr ?? ''}` };
  }
}

// Bundle a consumer that imports the package with webpack, peers left external:
// only the package entry resolution is under test. webpack (a devDependency of
// this package) is junction-linked into the consumer so the consumer dir stays
// hermetic.
function webpackConsume(dir, pkgRoot) {
  fs.mkdirSync(path.join(dir, 'node_modules', '@myriaddreamin'), { recursive: true });
  fs.symlinkSync(
    pkgRoot,
    path.join(dir, 'node_modules', '@myriaddreamin', 'typst.react'),
    'junction',
  );
  for (const dep of ['webpack', 'webpack-cli']) {
    fs.symlinkSync(path.join(pkgDir, 'node_modules', dep), path.join(dir, 'node_modules', dep), 'junction');
  }
  fs.writeFileSync(
    path.join(dir, 'entry.js'),
    "import { TypstDocument } from '@myriaddreamin/typst.react';\nconsole.log(TypstDocument);\n",
  );
  fs.writeFileSync(
    path.join(dir, 'webpack.config.cjs'),
    `module.exports = {
  mode: 'production',
  entry: './entry.js',
  output: { filename: 'bundle.js', path: __dirname + '/out' },
  externals: [/^react(\\/.*)?$/, /^@myriaddreamin\\/(?!typst\\.react$).*$/],
};\n`,
  );
  return run(process.execPath, [path.join(dir, 'node_modules', 'webpack', 'bin', 'webpack.js'), '--config', 'webpack.config.cjs'], dir);
}

test('webpack consumer: fails on published 0.8.0-rc3, passes with the new exports map', async t => {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'typst-react-764-'));
  t.after(() => fs.rmSync(tmp, { recursive: true, force: true }));

  const pack = run(npmCmd, ['pack', '@myriaddreamin/typst.react@0.8.0-rc3', '--pack-destination', tmp], tmp);
  assert.equal(pack.code, 0, `npm pack failed: ${pack.output}`);
  const tarball = path.join(
    tmp,
    fs.readdirSync(tmp).find(f => f.endsWith('.tgz')),
  );
  const pkgRoot = path.join(tmp, 'pkg');
  fs.mkdirSync(pkgRoot);
  // Relative paths: GNU tar would read the drive letter in C:\… as a remote host.
  assert.equal(
    run('tar', ['-xzf', path.basename(tarball), '-C', 'pkg', '--strip-components=1'], tmp).code,
    0,
  );

  // Before the change: the entry re-exports the tsc output with `import './typst.css?inline'`.
  const before = webpackConsume(path.join(tmp, 'before'), pkgRoot);
  assert.notEqual(before.code, 0, 'expected the published entry to fail under webpack');
  assert.match(
    before.output,
    /typst\.css\?inline/,
    `expected a resolution error about ./typst.css?inline, got:\n${before.output}`,
  );

  // After the change: same published tree, but with this branch's exports map.
  const repoint = JSON.parse(fs.readFileSync(path.join(pkgRoot, 'package.json'), 'utf8'));
  repoint.exports = branchExports;
  fs.writeFileSync(path.join(pkgRoot, 'package.json'), JSON.stringify(repoint, null, 2));
  const after = webpackConsume(path.join(tmp, 'after'), pkgRoot);
  assert.equal(after.code, 0, `webpack failed with the new exports map:\n${after.output}`);
});
