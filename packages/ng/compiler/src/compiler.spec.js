import assert from 'node:assert/strict';
import { access, readFile } from 'node:fs/promises';
import { constants } from 'node:fs';
import { delimiter, join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { spawn } from 'node:child_process';

import { createCliCompiler } from './cli.mjs';
import { createCompiler } from './index.mjs';
import { createNodeCompiler } from './node.mjs';
import { createWasmCompiler } from './wasm.mjs';
import { createWasmWorkerCompiler } from './wasm-worker.mjs';

const root = fileURLToPath(new URL('../../../../', import.meta.url));
const compilerWasmPath = join(root, 'packages/compiler/pkg/typst_ts_web_compiler_bg.wasm');
const fontPath = join(root, 'assets/data/LibertinusSerif-Regular-subset.otf');
const fixture = {
  mainFileContent: '#set page(width: 120pt, height: 80pt, margin: 10pt)\nHello, typst!',
};

test('auto selects node in Node.js', async () => {
  const compiler = await createCompiler({ backend: 'auto' });
  assert.equal(compiler.backend, 'node');
});

test('node backend compiles stable SVG', async () => {
  const compiler = await createNodeCompiler();
  const svg = await compiler.plainSvg(fixture);
  assert.match(svg, /^<svg /);
  assert.match(svg, /<defs>/);
});

test('wasm backend compiles vector artifact', async () => {
  await access(compilerWasmPath, constants.R_OK);
  await access(fontPath, constants.R_OK);

  const compiler = await createWasmCompiler(await wasmTestOptions());

  const vector = await compiler.vector(fixture);
  assert.ok(vector instanceof Uint8Array);
  assert.ok(vector.byteLength > 0);
});

test('wasm backend can mutate font and package providers', async () => {
  await access(compilerWasmPath, constants.R_OK);
  await access(fontPath, constants.R_OK);

  const compiler = await createWasmCompiler(await wasmTestOptions());

  await compiler.setPackageProvider({
    resolve() {
      throw new Error('fixture does not import packages');
    },
  });
  await compiler.setFontProvider({
    fonts: new Uint8Array(await readFile(fontPath)),
    loadOptions: { assets: false },
  });

  const vector = await compiler.vector(fixture);
  assert.ok(vector instanceof Uint8Array);
  assert.ok(vector.byteLength > 0);
});

test('wasm backend can mutate access model', async () => {
  await access(compilerWasmPath, constants.R_OK);
  await access(fontPath, constants.R_OK);

  const { MemoryAccessModel } = await import('@myriaddreamin/typst.ts/fs/memory');
  const accessModel = new MemoryAccessModel();
  accessModel.insertFile(
    '/@memory/main.typ',
    new TextEncoder().encode(fixture.mainFileContent),
    new Date(0),
  );

  const compiler = await createWasmCompiler(await wasmTestOptions());
  await compiler.setAccessModel(accessModel);

  const vector = await compiler.vector({
    root: '/@memory',
    mainFilePath: '/@memory/main.typ',
  });
  assert.ok(vector instanceof Uint8Array);
  assert.ok(vector.byteLength > 0);
});

test('wasm-worker backend compiles vector artifact', async () => {
  await access(compilerWasmPath, constants.R_OK);
  await access(fontPath, constants.R_OK);

  const compiler = await createWasmWorkerCompiler(await wasmTestOptions());

  try {
    assert.equal(compiler.backend, 'wasm-worker');
    const vector = await compiler.vector(fixture);
    assert.ok(vector instanceof Uint8Array);
    assert.ok(vector.byteLength > 0);
  } finally {
    await compiler.terminate();
  }
});

test('wasm-worker backend can mutate structured font provider', async () => {
  await access(compilerWasmPath, constants.R_OK);
  await access(fontPath, constants.R_OK);

  const compiler = await createWasmWorkerCompiler(await wasmTestOptions());

  try {
    await compiler.setFontProvider({
      fonts: new Uint8Array(await readFile(fontPath)),
      loadOptions: { assets: false },
    });

    const vector = await compiler.vector(fixture);
    assert.ok(vector instanceof Uint8Array);
    assert.ok(vector.byteLength > 0);
  } finally {
    await compiler.terminate();
  }
});

test('cli defaults to official typst and typst-ts-cli for vector', async () => {
  const compiler = await createCliCompiler();
  assert.equal(compiler.command, 'typst');
  assert.equal(compiler.vectorCommand, 'typst-ts-cli');
});

test('official cli SVG matches node plain SVG', { skip: !(await hasCommand('typst')) }, async () => {
  const node = await createNodeCompiler();
  const cli = await createCliCompiler();

  const nodeSvg = await node.plainSvg(fixture);
  const cliSvg = await cli.plainSvg(fixture);

  assert.equal(cliSvg, nodeSvg);
});

test('typst-ts-cli vector backend returns an artifact', { skip: !(await hasCommand('typst-ts-cli')) }, async () => {
  const compiler = await createCliCompiler();
  const vector = await compiler.vector(fixture);

  assert.ok(vector instanceof Uint8Array);
  assert.ok(vector.byteLength > 0);
});

function hasCommand(command) {
  return new Promise(resolve => {
    const child = spawn(command, ['--version'], {
      env: {
        ...process.env,
        PATH: [join(root, 'target/dist'), join(root, 'target/debug'), process.env.PATH]
          .filter(Boolean)
          .join(delimiter),
      },
      stdio: 'ignore',
    });

    child.on('error', () => resolve(false));
    child.on('close', code => resolve(code === 0));
  });
}

async function wasmTestOptions() {
  return {
    initOptions: {
      getModule: () => readFile(compilerWasmPath),
    },
    fontProvider: {
      fonts: new Uint8Array(await readFile(fontPath)),
      loadOptions: { assets: false },
    },
  };
}
