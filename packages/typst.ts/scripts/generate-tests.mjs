// generate all tests for the renderer
import fs from 'fs';
import path from 'path';

const categories = [
  'foundations',
  'html',
  'introspection',
  'layout',
  'loading',
  'math',
  'model',
  'package',
  'pdf',
  'pdftags',
  'playground',
  'scripting',
  'styling',
  'symbols',
  'syntax',
  'text',
  'visualize',
  'lint',
  'viewers',
  'skyzh-cv',
];

const staleCategories = [
  ...categories,
  'bugs',
  'meta',
];

const corpusRoot = path.resolve('../../fuzzers/corpora');

const files = categories.flatMap(category => {
  const dir = path.join(corpusRoot, category);
  return collectArtifactFiles(dir);
}).sort();

console.log(files);
const templateContent = fs.readFileSync('tests/test-template.mts', 'utf8');

const points = [];

for (const category of staleCategories) {
  fs.rmSync(path.join('tests', category), { recursive: true, force: true });
}

// generate all tests for the renderer
for (const file of files) {
  const testName = path.relative(corpusRoot, file).replace(/\\/g, "/").replace(/\.artifact\.sir\.in$/g, '');
  const testPath = `tests/${testName}.browser.test.mts`;
  const testDir = path.resolve(path.dirname(testPath));
  const artifactImport = toImportSpecifier(path.relative(testDir, file)) + '?url&inline';
  const testMainImport = toImportSpecifier(path.relative(testDir, path.resolve('tests/test-main.mjs')));

  const testContent = templateContent
    .replace('../../../../fuzzers/corpora/layout/transform-layout_02.artifact.sir.in?url&inline', artifactImport)
    .replace('./test-main.mjs', testMainImport)
    .replace(/layout\/transform-layout_02/g, testName);

  console.log(`Generating test for ${testPath}....`);
  fs.mkdirSync(path.dirname(testPath), { recursive: true });
  fs.writeFileSync(testPath, testContent);
  points.push(testName);
}

fs.writeFileSync('refs/test-points.json', JSON.stringify(points, null, 2));
console.log(`Generated ${points.length} tests.`);

function collectArtifactFiles(dir) {
  if (!fs.existsSync(dir)) {
    return [];
  }

  const entries = fs.readdirSync(dir, { withFileTypes: true });
  return entries.flatMap(entry => {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      return collectArtifactFiles(file);
    }
    return entry.isFile() && entry.name.endsWith('.artifact.sir.in') ? [file] : [];
  });
}

function toImportSpecifier(file) {
  const specifier = file.replace(/\\/g, '/');
  return specifier.startsWith('.') ? specifier : `./${specifier}`;
}
