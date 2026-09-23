// Test ④ for proposal #5 (typst.ts#764): type regression — a TS consumer that
// imports the package through its `exports["."].types` entry must type-check,
// and `TypstDocument` must stay the same exported component.
//
// Run via `pnpm test` in packages/typst.react (build runs first).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const pkgDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workDir = path.join(pkgDir, 'test', '.tmp-types');

test('types: a TS consumer resolves and type-checks the package entry', t => {
  fs.rmSync(workDir, { recursive: true, force: true });
  t.after(() => fs.rmSync(workDir, { recursive: true, force: true }));
  fs.mkdirSync(path.join(workDir, 'node_modules', '@myriaddreamin'), { recursive: true });
  fs.symlinkSync(
    pkgDir,
    path.join(workDir, 'node_modules', '@myriaddreamin', 'typst.react'),
    'junction',
  );
  fs.writeFileSync(
    path.join(workDir, 'consumer.tsx'),
    `import * as React from 'react';
import { TypstDocument } from '@myriaddreamin/typst.react';
// The exported symbol is still the same React component, usable via JSX/createElement.
export const el = React.createElement(TypstDocument, { artifact: new Uint8Array(0) });
export type Props = React.ComponentProps<typeof TypstDocument>;
`,
  );
  fs.writeFileSync(
    path.join(workDir, 'tsconfig.json'),
    JSON.stringify({
      compilerOptions: {
        noEmit: true,
        strict: true,
        target: 'es2020',
        lib: ['es2020', 'dom', 'dom.iterable'],
        skipLibCheck: true,
        jsx: 'react-jsx',
        module: 'esnext',
        moduleResolution: 'bundler',
        // resolve react/@types/react from the workspace install
        typeRoots: [path.join(pkgDir, 'node_modules', '@types')],
        paths: { react: [path.join(pkgDir, 'node_modules', '@types', 'react')] },
      },
      files: ['consumer.tsx'],
    }),
  );

  try {
    execFileSync('npx', ['tsc', '-p', 'tsconfig.json'], {
      cwd: workDir,
      encoding: 'utf8',
      shell: true,
    });
  } catch (e) {
    assert.fail(`type regression: consumer failed to type-check:\n${e.stdout}${e.stderr}`);
  }
});
