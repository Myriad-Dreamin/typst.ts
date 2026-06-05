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
const refsDir = process.argv[2]
  ? path.resolve(process.cwd(), process.argv[2])
  : path.join(packageRoot, 'refs');
const testPointsPath = path.join(refsDir, 'test-points.json');
const rendererDir = path.join(refsDir, 'renderer');
const skippedTests = new Set(browserSkippedTests);
const upstreamSuiteDir = resolveUpstreamSuiteDir(packageRoot);

const points = JSON.parse(fs.readFileSync(testPointsPath, 'utf8'));

if (!Array.isArray(points) || points.some(point => typeof point !== 'string')) {
  throw new Error(`${testPointsPath} must contain an array of test point strings`);
}

const upstreamRenderPoints = collectUpstreamRenderPoints(upstreamSuiteDir);

const getMissingImages = point => {
  const missing = [];
  if (!fs.existsSync(path.join(rendererDir, `${point}.svg.png`))) {
    missing.push('svg');
  }
  if (!fs.existsSync(path.join(rendererDir, `${point}.canvas.png`))) {
    missing.push('canvas');
  }
  return missing;
};

const renderedPoints = [];
const skippedMissingPoints = [];
const unexpectedMissingPoints = [];

for (const point of points) {
  const missingImages = getMissingImages(point);
  if (missingImages.length === 0) {
    renderedPoints.push(point);
    continue;
  }

  const isUpstreamRenderPoint = upstreamRenderPoints.has(point);
  const isFullyMissing = missingImages.length === 2;
  if (!isUpstreamRenderPoint && isFullyMissing && skippedTests.has(point)) {
    skippedMissingPoints.push(point);
    continue;
  }

  unexpectedMissingPoints.push({ point, missingImages, isUpstreamRenderPoint });
}

if (unexpectedMissingPoints.length > 0) {
  const details = unexpectedMissingPoints
    .slice(0, 100)
    .map(({ point, missingImages, isUpstreamRenderPoint }) => {
      const reason = isUpstreamRenderPoint
        ? 'expected by Typst upstream render ground truth'
        : 'not allowed by the browser skip list';
      return `  - ${point}: missing ${missingImages.join(', ')} (${reason})`;
    })
    .join('\n');
  const omitted = unexpectedMissingPoints.length > 100
    ? `\n  ... and ${unexpectedMissingPoints.length - 100} more`
    : '';

  throw new Error(
    `Vitest renderer refs are incomplete for ${unexpectedMissingPoints.length} point(s).\n`
      + `${details}${omitted}\n`
      + 'Each indexed render test point must produce both svg.png and canvas.png; '
      + 'only fully missing browser-skipped points may be omitted.',
  );
}

const nextContent = `${JSON.stringify(renderedPoints, null, 2)}\n`;
const previousContent = fs.readFileSync(testPointsPath, 'utf8');

if (previousContent !== nextContent) {
  fs.writeFileSync(testPointsPath, nextContent);
}

console.log(
  `Prepared ${renderedPoints.length}/${points.length} rendered Vitest ref points for ${path.relative(process.cwd(), testPointsPath)}`,
);

console.log(
  `Validated ${upstreamRenderPoints.size} Typst upstream render point(s) from ${path.relative(process.cwd(), upstreamSuiteDir)}`,
);

if (skippedMissingPoints.length > 0) {
  console.log(
    `Omitted ${skippedMissingPoints.length} fully missing point(s) because their browser tests are explicitly skipped.`,
  );
}
