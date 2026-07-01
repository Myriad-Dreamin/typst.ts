import { mainPathOf, normalizeInput, peerError, unsupported } from './shared.mjs';

export async function createWasmCompiler(options = {}) {
  let mod;
  try {
    mod = await import('@myriaddreamin/typst.ts/compiler');
  } catch (error) {
    throw peerError('wasm', '@myriaddreamin/typst.ts', error);
  }

  const inner = mod.createTypstCompiler();
  await inner.init(options.wasm?.initOptions || options.initOptions);
  return new WasmCompilerFacade(inner, mod.CompileFormatEnum);
}

export class WasmCompilerFacade {
  backend = 'wasm';

  constructor(inner, formatEnum) {
    this.inner = inner;
    this.formatEnum = formatEnum;
  }

  async compile(input) {
    return this.vector(input);
  }

  async vector(input) {
    const opts = await this.prepareInput(input);
    return unwrapResult(await this.inner.compile({
      ...opts,
      format: this.formatEnum.vector,
    }));
  }

  async pdf(input) {
    const opts = await this.prepareInput(input);
    return unwrapResult(await this.inner.compile({
      ...opts,
      format: this.formatEnum.pdf,
    }));
  }

  plainSvg() {
    unsupported('wasm', 'plainSvg');
  }

  svg() {
    unsupported('wasm', 'svg');
  }

  html() {
    unsupported('wasm', 'html');
  }

  async query(input, options) {
    return this.inner.query({ ...options, ...(await this.prepareInput(input)) });
  }

  addSource(path, source) {
    return this.inner.addSource(path, source);
  }

  mapShadow(path, content) {
    return this.inner.mapShadow(path, content);
  }

  unmapShadow(path) {
    return this.inner.unmapShadow(path);
  }

  resetShadow() {
    return this.inner.resetShadow();
  }

  reset() {
    return this.inner.reset();
  }

  async prepareInput(input) {
    const opts = normalizeInput(input);
    const mainFilePath = mainPathOf(opts);

    if (opts.mainFileContent != null) {
      this.inner.addSource(mainFilePath, opts.mainFileContent);
    }

    return {
      root: opts.root || opts.workspace,
      mainFilePath,
      inputs: opts.inputs,
      diagnostics: opts.diagnostics,
    };
  }
}

export { createWasmCompiler as createCompiler };

function unwrapResult(value) {
  return value && typeof value === 'object' && 'result' in value ? value.result : value;
}
