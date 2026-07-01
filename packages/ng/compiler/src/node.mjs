import { normalizeInput, peerError } from './shared.mjs';

export async function createNodeCompiler(options = {}) {
  let mod;
  try {
    mod = await import('@myriaddreamin/typst-ts-node-compiler');
  } catch (error) {
    throw peerError('node', '@myriaddreamin/typst-ts-node-compiler', error);
  }

  return new NodeCompilerFacade(mod.NodeCompiler.create(options.node || options));
}

export class NodeCompilerFacade {
  backend = 'node';

  constructor(inner) {
    this.inner = inner;
  }

  compile(input) {
    return Promise.resolve(this.inner.compile(normalizeInput(input)));
  }

  compileHtml(input) {
    return Promise.resolve(this.inner.compileHtml(normalizeInput(input)));
  }

  vector(input) {
    return Promise.resolve(this.inner.vector(normalizeInput(input))).then(toUint8Array);
  }

  pdf(input, options) {
    return Promise.resolve(this.inner.pdf(normalizeInput(input), options)).then(toUint8Array);
  }

  plainSvg(input) {
    return Promise.resolve(this.inner.plainSvg(normalizeInput(input)));
  }

  svg(input) {
    return Promise.resolve(this.inner.svg(normalizeInput(input)));
  }

  html(input) {
    return Promise.resolve(this.inner.html(normalizeInput(input)));
  }

  query(input, options) {
    return Promise.resolve(this.inner.query(normalizeInput(input), options));
  }

  addSource(path, source) {
    return this.inner.addSource(path, source);
  }

  mapShadow(path, content) {
    return this.inner.mapShadow(path, toBuffer(content));
  }

  unmapShadow(path) {
    return this.inner.unmapShadow(path);
  }

  resetShadow() {
    return this.inner.resetShadow();
  }

  evictCache(maxAge = 10) {
    return this.inner.evictCache(maxAge);
  }
}

function toUint8Array(value) {
  return value instanceof Uint8Array
    ? new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
    : new Uint8Array(value);
}

function toBuffer(value) {
  if (typeof Buffer === 'undefined') {
    return value;
  }
  return Buffer.isBuffer(value)
    ? value
    : Buffer.from(value.buffer, value.byteOffset, value.byteLength);
}

export { createNodeCompiler as createCompiler };
