// generate all tests for the renderer
import fs from 'fs';

// glob all artifact.sir.in files
const files = fs.globSync('../../fuzzers/corpora/{skyzh-cv,layout}/*.sir.in', {
  eager: true,
  query: '?url&inline',
  import: 'default',
});
console.log(files);
const templateContent = fs.readFileSync('tests/test-template.mts', 'utf8')
    .replace(/\.\/test-main.mjs/g, "../../tests/test-main.mjs");

const points = [];

// generate all tests for the renderer
for (const file of files) {
  const testName = file.replace(/\\/g, "/").replace("../../fuzzers/corpora/", "").replace(/\.artifact\.sir\.in$/g, '');

  const testContent = templateContent.replace(/layout\/transform-layout_02/g, testName);

  console.log(`Generating test for tests/${testName}....`);
  fs.mkdirSync(`tests/${testName}/..`, { recursive: true });
  fs.writeFileSync(`tests/${testName}.browser.test.mts`, testContent);
  points.push(testName);
}

fs.writeFileSync('refs/test-points.json', JSON.stringify(points, null, 2));
