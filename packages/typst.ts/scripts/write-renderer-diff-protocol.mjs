import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import pngjs from 'pngjs';
import { resolveUpstreamSuiteDir } from './typst-upstream-render-points.mjs';

const { PNG } = pngjs;

const HASH_BITS = 16;
const ARTIFACT_NAME = 'renderer-diff-vitest';
const MANIFEST_FILE = 'renderer-diff-manifest.json';
const OFFICIAL_GROUP = 'official';
const CANVAS_GROUP = 'canvas';
const SVG_GROUP = 'svg';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageRoot = path.resolve(scriptDir, '..');
const workspaceRoot = path.resolve(packageRoot, '../..');
const refsDir = process.argv[2]
  ? path.resolve(process.cwd(), process.argv[2])
  : path.join(packageRoot, 'refs');
const outputRoot = process.argv[3]
  ? path.resolve(process.cwd(), process.argv[3])
  : path.join(workspaceRoot, 'target/typst-artifacts', ARTIFACT_NAME);

const testPointsPath = path.join(refsDir, 'test-points.json');
const rendererDir = path.join(refsDir, 'renderer');
const upstreamSuiteDir = resolveUpstreamSuiteDir(packageRoot);
const upstreamRefDir = path.resolve(upstreamSuiteDir, '../ref');
const upstreamRenderRefDir = path.join(upstreamRefDir, 'render');

if (!fs.existsSync(upstreamRefDir)) {
  throw new Error(
    `Typst upstream ref directory was not found at ${upstreamRefDir}. ` +
      'Set TYPST_TS_UPSTREAM_SUITE_DIR to the typst/tests/suite checkout used as ground truth.',
  );
}

const points = JSON.parse(fs.readFileSync(testPointsPath, 'utf8'));
if (!Array.isArray(points) || points.some(point => typeof point !== 'string')) {
  throw new Error(`${testPointsPath} must contain an array of test point strings`);
}

fs.rmSync(outputRoot, { recursive: true, force: true });
fs.mkdirSync(outputRoot, { recursive: true });

const cases = [];
let withOfficial = 0;
let withoutOfficial = 0;

for (const rawPoint of points) {
  const point = normalizePoint(rawPoint);
  const canvasPath = path.join(rendererDir, `${point}.canvas.png`);
  const svgPath = path.join(rendererDir, `${point}.svg.png`);

  if (!fs.existsSync(canvasPath) || !fs.existsSync(svgPath)) {
    const missing = [
      fs.existsSync(canvasPath) ? undefined : 'canvas',
      fs.existsSync(svgPath) ? undefined : 'svg',
    ].filter(Boolean);
    throw new Error(`${point}: missing rendered ${missing.join(', ')} PNG(s) in ${rendererDir}`);
  }

  const assets = {};
  const images = {};
  const officialPath = officialRefPath(point);

  if (officialPath) {
    const official = writeAsset(OFFICIAL_GROUP, point, officialPath);
    assets[OFFICIAL_GROUP] = official.asset;
    images[OFFICIAL_GROUP] = official.image;
    withOfficial += 1;
  } else {
    withoutOfficial += 1;
  }

  const canvas = writeAsset(CANVAS_GROUP, point, canvasPath);
  assets[CANVAS_GROUP] = canvas.asset;
  images[CANVAS_GROUP] = canvas.image;

  const svg = writeAsset(SVG_GROUP, point, svgPath);
  assets[SVG_GROUP] = svg.asset;
  images[SVG_GROUP] = svg.image;

  const comparisons = [];
  if (images[OFFICIAL_GROUP]) {
    comparisons.push(compareGroups(OFFICIAL_GROUP, CANVAS_GROUP, images, assets));
    comparisons.push(compareGroups(OFFICIAL_GROUP, SVG_GROUP, images, assets));
  }
  comparisons.push(compareGroups(CANVAS_GROUP, SVG_GROUP, images, assets));

  cases.push({
    name: point,
    status: statusFromComparisons(comparisons),
    assets,
    comparisons,
  });
}

const manifest = {
  schemaVersion: 1,
  artifactName: ARTIFACT_NAME,
  groups: [
    {
      id: OFFICIAL_GROUP,
      label: 'Typst official renderer',
      kind: 'baseline',
      source: 'typst/typst tests/ref',
    },
    {
      id: CANVAS_GROUP,
      label: 'typst.ts canvas',
      kind: 'renderer',
    },
    {
      id: SVG_GROUP,
      label: 'typst.ts svg',
      kind: 'renderer',
    },
  ],
  source: {
    suite: 'typst.ts browser renderer suite',
    typstTests: upstreamSuiteDir,
    ...(process.env.TYPST_TS_UPSTREAM_REF ? { typstRef: process.env.TYPST_TS_UPSTREAM_REF } : {}),
    ...(process.env.GITHUB_RUN_ID ? { githubRunId: process.env.GITHUB_RUN_ID } : {}),
    ...(process.env.GITHUB_SHA ? { githubSha: process.env.GITHUB_SHA } : {}),
  },
  hash: {
    algorithm: 'blockhash',
    bits: HASH_BITS,
    format: 'ihash16:<hex>',
    distance: 'hamming',
  },
  summary: summaryFromCases(cases),
  cases,
};

fs.writeFileSync(path.join(outputRoot, MANIFEST_FILE), `${JSON.stringify(manifest, null, 2)}\n`);

console.log(
  `Prepared ${ARTIFACT_NAME} with ${cases.length} case(s) at ${path.relative(process.cwd(), outputRoot)}`,
);
console.log(
  `Included ${withOfficial} case(s) with official Typst refs and ${withoutOfficial} case(s) without official refs.`,
);

function normalizePoint(point) {
  if (point.includes('\\')) {
    throw new Error(`invalid test point ${point}: backslashes are not allowed`);
  }

  const parts = point.split('/');
  if (parts.some(part => part === '' || part === '.' || part === '..')) {
    throw new Error(`invalid test point ${point}: path traversal is not allowed`);
  }

  return parts.join('/');
}

function officialRefPath(point) {
  const name = point.split('/').at(-1);
  const candidate = path.join(upstreamRenderRefDir, `${name}.png`);
  return fs.existsSync(candidate) ? candidate : undefined;
}

function writeAsset(group, point, sourcePath) {
  const bytes = fs.readFileSync(sourcePath);
  const image = PNG.sync.read(bytes);
  const perceptualHash = `ihash16:${blockHash(image, HASH_BITS)}`;
  const sha256Digest = `sha256:${createHash('sha256').update(bytes).digest('hex')}`;

  const pngPath = groupAssetPath(group, point, 'png');
  const hashPath = groupAssetPath(group, point, 'hash');
  const sha256Path = groupAssetPath(group, point, 'sha256');

  fs.mkdirSync(path.dirname(pngPath), { recursive: true });
  fs.writeFileSync(pngPath, bytes);
  fs.writeFileSync(hashPath, `${perceptualHash}\n`);
  fs.writeFileSync(sha256Path, `${sha256Digest}\n`);

  return {
    asset: {
      png: groupRelPath(group, point, 'png'),
      hash: groupRelPath(group, point, 'hash'),
      sha256: groupRelPath(group, point, 'sha256'),
      width: image.width,
      height: image.height,
      perceptualHash,
      sha256Digest,
    },
    image,
  };
}

function groupAssetPath(group, point, extension) {
  return path.join(
    outputRoot,
    group,
    ...point.split('/').slice(0, -1),
    `${point.split('/').at(-1)}.${extension}`,
  );
}

function groupRelPath(group, point, extension) {
  return `${group}/${point}.${extension}`;
}

function compareGroups(lhs, rhs, images, assets) {
  const metrics = compareImages(
    images[lhs],
    images[rhs],
    assets[lhs].perceptualHash,
    assets[rhs].perceptualHash,
  );
  return {
    lhs,
    rhs,
    status: metrics.pixelMismatchCount === 0 ? 'matched' : 'different',
    metrics,
  };
}

function compareImages(lhs, rhs, lhsHash, rhsHash) {
  const width = Math.max(lhs.width, rhs.width, 1);
  const height = Math.max(lhs.height, rhs.height, 1);
  const totalPixels = width * height;
  let pixelMismatchCount = 0;
  let maxChannelDelta = 0;
  let totalAbsDelta = 0;

  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const left = samplePixel(lhs, x, y);
      const right = samplePixel(rhs, x, y);
      let pixelDelta = 0;

      for (let channel = 0; channel < 3; channel += 1) {
        const delta = Math.abs(left[channel] - right[channel]);
        pixelDelta = Math.max(pixelDelta, delta);
        maxChannelDelta = Math.max(maxChannelDelta, delta);
        totalAbsDelta += delta;
      }

      if (pixelDelta > 0 || left[3] !== right[3]) {
        pixelMismatchCount += 1;
      }
    }
  }

  return {
    perceptualHashDistance: perceptualHashDistance(lhsHash, rhsHash),
    pixelMismatchCount,
    pixelMismatchRatio: pixelMismatchCount / totalPixels,
    meanAbsoluteError: totalAbsDelta / (totalPixels * 3 * 255),
    maxChannelDelta,
  };
}

function samplePixel(image, x, y) {
  if (x >= image.width || y >= image.height) {
    return [0, 0, 0, 0];
  }

  const offset = (y * image.width + x) * 4;
  return [
    image.data[offset],
    image.data[offset + 1],
    image.data[offset + 2],
    image.data[offset + 3],
  ];
}

function perceptualHashDistance(lhs, rhs) {
  const left = stripHashPrefix(lhs);
  const right = stripHashPrefix(rhs);
  const len = Math.min(left.length, right.length);
  let distance = Math.abs(left.length - right.length) * 4;

  for (let index = 0; index < len; index += 1) {
    const l = Number.parseInt(left[index], 16);
    const r = Number.parseInt(right[index], 16);
    distance += popCount((Number.isNaN(l) ? 0 : l) ^ (Number.isNaN(r) ? 0 : r));
  }

  return distance;
}

function stripHashPrefix(value) {
  return value.replace(/^ihash16:/, '').split('?')[0];
}

function popCount(value) {
  let count = 0;
  let rest = value;
  while (rest > 0) {
    count += rest & 1;
    rest >>= 1;
  }
  return count;
}

function statusFromComparisons(comparisons) {
  return comparisons.some(comparison => comparison.status !== 'matched') ? 'different' : 'matched';
}

function summaryFromCases(items) {
  const summary = {
    total: items.length,
    matched: 0,
    different: 0,
    renderErrors: 0,
  };

  for (const item of items) {
    if (item.status === 'matched') {
      summary.matched += 1;
    } else if (item.status === 'render-error') {
      summary.renderErrors += 1;
    } else {
      summary.different += 1;
    }
  }

  return summary;
}

function blockHash(image, bits) {
  const width = image.width;
  const height = image.height;
  const evenX = width % bits === 0;
  const evenY = height % bits === 0;
  const blocks = evenX && evenY ? blockHashEven(image, bits) : blockHashPrecise(image, bits);

  translateBlocksToBits(blocks, (width * height) / (bits * bits));
  return bitsToHex(blocks);
}

function blockHashEven(image, bits) {
  const blockSizeX = image.width / bits;
  const blockSizeY = image.height / bits;
  const result = [];

  for (let y = 0; y < bits; y += 1) {
    for (let x = 0; x < bits; x += 1) {
      let total = 0;
      for (let iy = 0; iy < blockSizeY; iy += 1) {
        for (let ix = 0; ix < blockSizeX; ix += 1) {
          total += pixelValue(image, x * blockSizeX + ix, y * blockSizeY + iy);
        }
      }
      result.push(total);
    }
  }

  return result;
}

function blockHashPrecise(image, bits) {
  const width = image.width;
  const height = image.height;
  const blockWidth = width / bits;
  const blockHeight = height / bits;
  const evenX = width % bits === 0;
  const evenY = height % bits === 0;
  const blocks = Array.from({ length: bits }, () => Array.from({ length: bits }, () => 0));

  for (let y = 0; y < height; y += 1) {
    const [blockTop, blockBottom, weightTop, weightBottom] = blockPosition(
      y,
      height,
      blockHeight,
      evenY,
    );
    for (let x = 0; x < width; x += 1) {
      const [blockLeft, blockRight, weightLeft, weightRight] = blockPosition(
        x,
        width,
        blockWidth,
        evenX,
      );
      const value = pixelValue(image, x, y);

      blocks[blockTop][blockLeft] += value * weightTop * weightLeft;
      blocks[blockTop][blockRight] += value * weightTop * weightRight;
      blocks[blockBottom][blockLeft] += value * weightBottom * weightLeft;
      blocks[blockBottom][blockRight] += value * weightBottom * weightRight;
    }
  }

  return blocks.flat();
}

function blockPosition(pos, max, blockSize, even) {
  if (even) {
    const block = Math.floor(pos / blockSize);
    return [block, block, 1, 0];
  }

  const posMod = (pos + 1) % blockSize;
  const posFrac = posMod - Math.floor(posMod);
  const posInt = posMod - posFrac;
  const weightFirst = 1 - posFrac;
  const weightSecond = posFrac;

  if (posInt > 0 || pos + 1 === max) {
    const block = Math.floor(pos / blockSize);
    return [block, block, weightFirst, weightSecond];
  }

  return [Math.floor(pos / blockSize), Math.ceil(pos / blockSize), weightFirst, weightSecond];
}

function translateBlocksToBits(blocks, pixelsPerBlock) {
  const halfBlockValue = (pixelsPerBlock * 256 * 3) / 2;
  const bandSize = blocks.length / 4;

  for (let band = 0; band < 4; band += 1) {
    const start = band * bandSize;
    const end = start + bandSize;
    const medianValue = median(blocks.slice(start, end));
    for (let index = start; index < end; index += 1) {
      const value = blocks[index];
      blocks[index] =
        value > medianValue || (Math.abs(value - medianValue) < 1 && medianValue > halfBlockValue)
          ? 1
          : 0;
    }
  }
}

function median(values) {
  const sorted = [...values].sort((lhs, rhs) => lhs - rhs);
  if (sorted.length % 2 === 0) {
    return (sorted[sorted.length / 2] + sorted[sorted.length / 2 + 1]) / 2;
  }
  return sorted[Math.floor(sorted.length / 2)];
}

function bitsToHex(bits) {
  let hex = '';
  for (let index = 0; index < bits.length; index += 4) {
    let value = 0;
    for (const bit of bits.slice(index, index + 4)) {
      value = (value << 1) | (bit === 0 ? 0 : 1);
    }
    hex += value.toString(16);
  }
  return hex;
}

function pixelValue(image, x, y) {
  const offset = (y * image.width + x) * 4;
  if (image.data[offset + 3] === 0) {
    return 765;
  }
  return image.data[offset] + image.data[offset + 1] + image.data[offset + 2];
}
