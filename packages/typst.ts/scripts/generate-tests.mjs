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
  return collectArtifactFiles(dir).map(file => path.relative('.', file).replace(/\\/g, '/'));
}).sort();

console.log(files);
const templateContent = fs.readFileSync('tests/test-template.mts', 'utf8')
    .replace(/\.\/test-main.mjs/g, "../../tests/test-main.mjs");

const points = [];

for (const category of staleCategories) {
  fs.rmSync(path.join('tests', category), { recursive: true, force: true });
}

// generate all tests for the renderer
for (const file of files) {
  const testName = file.replace(/\\/g, "/").replace("../../fuzzers/corpora/", "").replace(/\.artifact\.sir\.in$/g, '');

  const testContent = templateContent.replace(/layout\/transform-layout_02/g, testName);

  console.log(`Generating test for tests/${testName}....`);
  fs.mkdirSync(path.dirname(`tests/${testName}.browser.test.mts`), { recursive: true });
  fs.writeFileSync(`tests/${testName}.browser.test.mts`, testContent);
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
