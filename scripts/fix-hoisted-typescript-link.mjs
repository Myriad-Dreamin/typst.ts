import { existsSync, lstatSync, mkdirSync, realpathSync, rmSync, symlinkSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const rootNodeModules = join(projectRoot, 'node_modules');
const hoistedTypeScript = join(rootNodeModules, 'typescript');
const nestedNodeModules = join(rootNodeModules, 'node_modules');
const nestedTypeScript = join(nestedNodeModules, 'typescript');

if (process.platform !== 'win32') {
  process.exit(0);
}

if (!existsSync(hoistedTypeScript)) {
  console.warn('[postinstall] skipping TypeScript link fix because node_modules/typescript is missing.');
  process.exit(0);
}

mkdirSync(nestedNodeModules, { recursive: true });

if (existsSync(nestedTypeScript)) {
  try {
    if (realpathSync(nestedTypeScript) === realpathSync(hoistedTypeScript)) {
      process.exit(0);
    }
  } catch {
    // Fall through and recreate broken links below.
  }

  if (lstatSync(nestedTypeScript).isSymbolicLink()) {
    rmSync(nestedTypeScript, { force: true, recursive: true });
  } else {
    console.warn(
      '[postinstall] skipping TypeScript link fix because node_modules/node_modules/typescript already exists.',
    );
    process.exit(0);
  }
}

symlinkSync(hoistedTypeScript, nestedTypeScript, 'junction');
console.log('[postinstall] linked node_modules/node_modules/typescript to the hoisted TypeScript install.');
