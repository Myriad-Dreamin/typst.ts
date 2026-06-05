import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { browserSkippedTests } from './browser-skipped-tests.mjs';
import {
  collectUpstreamRenderPoints,
  resolveUpstreamSuiteDir,
} from './typst-upstream-render-points.mjs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(scriptDir, '..');
const testPointsPath = path.join(packageRoot, 'refs/test-points.json');
const upstreamSuiteDir = resolveUpstreamSuiteDir(packageRoot);

const points = JSON.parse(fs.readFileSync(testPointsPath, 'utf8'));
if (!Array.isArray(points) || points.some(point => typeof point !== 'string')) {
  throw new Error(`${testPointsPath} must contain an array of test point strings`);
}

const pointSet = new Set(points);
const upstreamRenderPoints = collectUpstreamRenderPoints(upstreamSuiteDir);
const tests = browserSkippedTests
  .filter(point => pointSet.has(point) && upstreamRenderPoints.has(point))
  .map(point => `tests/${point}.browser.test.mts`);

if (tests.length === 0) {
  console.log('No skipped Typst upstream render refs need to be generated.');
  process.exit(0);
}

console.log(
  `Rendering ${tests.length} skipped Typst upstream ref test(s) from ${path.relative(process.cwd(), upstreamSuiteDir)}`,
);

const result = spawnSync(
  'yarn',
  ['vitest', 'run', ...tests, '--project', 'browser', '--update'],
  {
    cwd: packageRoot,
    env: {
      ...process.env,
      TYPST_TS_INCLUDE_SKIPPED_BROWSER_TESTS: '1',
    },
    stdio: 'inherit',
    shell: process.platform === 'win32',
  },
);

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);
