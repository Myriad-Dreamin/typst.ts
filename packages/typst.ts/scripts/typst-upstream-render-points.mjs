import fs from 'node:fs';
import path from 'node:path';

export function resolveUpstreamSuiteDir(packageRoot) {
  return process.env.TYPST_TS_UPSTREAM_SUITE_DIR
    ? path.resolve(process.cwd(), process.env.TYPST_TS_UPSTREAM_SUITE_DIR)
    : path.resolve(packageRoot, '../../../typst/tests/suite');
}

export function collectUpstreamRenderPoints(suiteDir) {
  if (!fs.existsSync(suiteDir)) {
    throw new Error(
      `Typst upstream test suite was not found at ${suiteDir}. `
        + 'Set TYPST_TS_UPSTREAM_SUITE_DIR to the typst/tests/suite checkout used as ground truth.',
    );
  }

  const points = new Set();
  for (const file of collectTypFiles(suiteDir)) {
    const text = fs.readFileSync(file, 'utf8');
    if (text.startsWith('// SKIP') || !text.split(/\r?\n/).some(line => line.startsWith('---'))) {
      continue;
    }

    for (const section of parseSections(text)) {
      const point = toCorpusPoint(suiteDir, file, section.name);
      if (
        !isRenderTarget(section)
        || hasExpectedError(section)
        || isUnsupportedForCorpus(section, point)
      ) {
        continue;
      }

      points.add(point);
    }
  }

  return points;
}

function collectTypFiles(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  return entries.flatMap(entry => {
    const file = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      return collectTypFiles(file);
    }
    return entry.isFile() && entry.name.endsWith('.typ') ? [file] : [];
  });
}

function parseSections(text) {
  const sections = [];
  let current = null;

  for (const match of text.matchAll(/^---.*$/gm)) {
    const header = parseHeader(match[0]);
    if (!header) {
      continue;
    }

    if (current) {
      sections.push({
        ...current,
        body: trimTrailingNewlines(text.slice(current.bodyStart, match.index)),
      });
    }

    current = {
      ...header,
      bodyStart: nextLineStart(text, match.index + match[0].length),
    };
  }

  if (current) {
    sections.push({
      ...current,
      body: trimTrailingNewlines(text.slice(current.bodyStart)),
    });
  }

  return sections;
}

function parseHeader(line) {
  const trimmed = line.trimEnd();
  if (!trimmed.startsWith('---') || !trimmed.endsWith('---')) {
    return undefined;
  }

  const content = trimmed.slice(3, -3).trim();
  const words = content.split(/\s+/).filter(Boolean);
  if (words.length === 0) {
    return undefined;
  }

  return {
    name: words[0],
    attrs: words.slice(1),
  };
}

function nextLineStart(text, offset) {
  if (text[offset] === '\r' && text[offset + 1] === '\n') {
    return offset + 2;
  }
  if (text[offset] === '\n') {
    return offset + 1;
  }
  return offset;
}

function trimTrailingNewlines(text) {
  return text.replace(/[\r\n]+$/g, '');
}

function isRenderTarget(section) {
  const hasRender = section.attrs.includes('render');
  const hasNonRender = section.attrs.some(attr => attr === 'html' || attr === 'pdftags' || attr === 'bundle');
  return hasRender || !hasNonRender;
}

function hasExpectedError(section) {
  return section.body
    .split(/\r?\n/)
    .some(line => line.trimStart().startsWith('// Error:'));
}

function isUnsupportedForCorpus(section, point) {
  return section.body.includes('@test/')
    || section.body.includes('read("./eval.typ")')
    || point === 'foundations/datetime-display'
    || point === 'foundations/path'
    || point.startsWith('layout/inline/baseline-');
}

function toCorpusPoint(suiteDir, file, name) {
  const relative = path.relative(suiteDir, file).replace(/\\/g, '/');
  const components = relative.split('/');
  if (components.length === 1) {
    return `${path.basename(components[0], '.typ')}/${name}`;
  }

  components[components.length - 1] = name;
  return components.join('/');
}
