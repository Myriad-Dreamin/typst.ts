import * as resolve from 'resolve.exports';
import { readFileSync, existsSync } from 'fs';
import { join } from 'path';

for (const pkgName of ['typst-all-in-one.ts', 'typst.ts', 'renderer', 'compiler', 'parser', 'typst.react', 'typst.vue3']) {
  const pkgDir = join(import.meta.dirname, '../..', pkgName);
  const pkgPath = join(pkgDir, 'package.json');
  const pkg = JSON.parse(readFileSync(pkgPath));
  if (!pkg.exports) {
    console.log(`No exports found for ${pkgName}`);
    continue;
  }
  const validExports = Object.keys(pkg.exports).filter(key => !key.includes('*'));

  for (const key of validExports) {
    for (const conditions of [
      // commonjs
      ['require'],
      // commonjs type declarations
      ['require', 'types'],
      // commonjs node
      ['require', 'node'],
      // commonjs browser
      ['require', 'browser'],
      // vite
      ['require', 'browser', 'production'],

      // esm
      ['module', 'import'],
      // esm type declarations
      ['module', 'import', 'types'],
      // esm node
      ['module', 'import', 'node'],
      // esm browser
      ['module', 'import', 'browser'],
      // vite
      ['module', 'import', 'browser', 'production'],
    ]) {
      for (const unsafe of [false, true]) {
        try {
          const exportPath = resolve.exports(pkg, key, { conditions, unsafe });
          if (conditions.includes('types') && exportPath && !exportPath[0].endsWith('ts')) {
            throw new Error(
              `This is not a types export for ${pkgName} ${key} with conditions ${conditions} and unsafe ${unsafe}:`,
            );
          }
          for (const p of exportPath || []) {
            const exportAbsPath = join(pkgDir, p);
            if (!existsSync(exportAbsPath)) {
              throw new Error(
                `This export does not exist in filesystem for ${pkgName} ${key} with conditions ${conditions} and unsafe ${unsafe}: ${p}`,
              );
            }
          }
        } catch (error) {
          console.error(
            `Error resolving export for ${pkgName} ${key} with conditions ${conditions} and unsafe ${unsafe}:`,
          );
          throw error;
        }
      }
    }
  }
}
