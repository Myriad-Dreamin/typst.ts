const { readFileSync, writeFileSync } = require('fs');
const { join } = require('path');

const { dirs } = require('./publish-targets');

const rootDir = join(__dirname, '..');

function readJson(relativePath) {
  return JSON.parse(readFileSync(join(rootDir, relativePath), 'utf8'));
}

function writeJson(relativePath, value) {
  writeFileSync(join(rootDir, relativePath), `${JSON.stringify(value, null, 2)}\n`);
}

const rootPackage = readJson('package.json');
const optionalDependencies = {};

for (const dir of dirs) {
  const packageJsonPath = `npm/${dir}/package.json`;
  const manifest = readJson(packageJsonPath);

  manifest.version = rootPackage.version;
  optionalDependencies[manifest.name] = rootPackage.version;

  writeJson(packageJsonPath, manifest);
}

rootPackage.optionalDependencies = optionalDependencies;
writeJson('package.json', rootPackage);

console.log(
  `Prepared ${dirs.length} platform packages for ${rootPackage.name}@${rootPackage.version}.`,
);
