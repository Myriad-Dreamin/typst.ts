import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(scriptDir, '..');
const refsDir = process.argv[2]
  ? path.resolve(process.cwd(), process.argv[2])
  : path.join(packageRoot, 'refs');
const testPointsPath = path.join(refsDir, 'test-points.json');
const rendererDir = path.join(refsDir, 'renderer');

const points = JSON.parse(fs.readFileSync(testPointsPath, 'utf8'));

if (!Array.isArray(points) || points.some(point => typeof point !== 'string')) {
  throw new Error(`${testPointsPath} must contain an array of test point strings`);
}

const hasRenderedImages = point => {
  return fs.existsSync(path.join(rendererDir, `${point}.svg.png`))
    && fs.existsSync(path.join(rendererDir, `${point}.canvas.png`));
};

const renderedPoints = points.filter(hasRenderedImages);
const nextContent = `${JSON.stringify(renderedPoints, null, 2)}\n`;
const previousContent = fs.readFileSync(testPointsPath, 'utf8');

if (previousContent !== nextContent) {
  fs.writeFileSync(testPointsPath, nextContent);
}

console.log(
  `Prepared ${renderedPoints.length}/${points.length} rendered Vitest ref points for ${path.relative(process.cwd(), testPointsPath)}`,
);
