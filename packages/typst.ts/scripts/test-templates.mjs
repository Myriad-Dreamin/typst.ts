import { readdirSync } from 'fs';

import { join } from 'path';
import { execSync } from 'child_process';

function yarn(cwd, command) {
  return execSync(`yarn ${command}`, {
    shell: true,
    cwd,
    stdio: 'inherit',
  });
}

function main() {
  const projectRoot = join(import.meta.dirname, '../../../');
  const templateRoot = join(projectRoot, 'templates');

  //   const templates = readdirSync(templateRoot);
  const templates = [
    ['compiler-node'],
    ['compiler-wasm-cjs'],
    ['compiler-wasm-esm'],
    ['renderer-wasm-cjs'],
    ['renderer-wasm-esm'],
    ['tsx'],
  ];

  for (const [templateName] of templates) {
    const templateDir = join(templateRoot, templateName);
    console.log(`start testing ${templateDir}`);
    yarn(templateDir, 'build');
    yarn(templateDir, 'test');
  }
}

console.log(main());
