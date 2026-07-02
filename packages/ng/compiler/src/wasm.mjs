import { mainPathOf, normalizeInput, peerError, unsupported } from './shared.mjs';

export async function createWasmCompiler(options = {}) {
  let mod;
  try {
    mod = await import('@myriaddreamin/typst.ts/compiler');
  } catch (error) {
    throw peerError('wasm', '@myriaddreamin/typst.ts', error);
  }

  return WasmCompiler.create(mod, options);
}

export class WasmCompiler {
  backend = 'wasm';

  static async create(mod, options = {}) {
    const compiler = new WasmCompiler(mod, options);
    await compiler.rebuild();
    return compiler;
  }

  constructor(mod, options = {}) {
    const wasmOptions = options.wasm || {};
    this.mod = mod;
    this.formatEnum = mod.CompileFormatEnum;
    this.baseInitOptions = wasmOptions.initOptions || options.initOptions;
    this.fontProvider = optionValue(options, wasmOptions, 'fontProvider');
    this.accessModel = optionValue(options, wasmOptions, 'accessModel');
    this.packageProvider =
      optionValue(options, wasmOptions, 'packageProvider') ??
      optionValue(options, wasmOptions, 'packageRegistry');
    this.shadowEntries = new Map();
  }

  async rebuild() {
    const inner = this.mod.createTypstCompiler();
    await inner.init(await this.createInitOptions());
    this.inner = inner;
    this.replayShadowEntries();
  }

  async createInitOptions() {
    const initOptions = { ...(this.baseInitOptions || {}) };
    const beforeBuild = [...(this.baseInitOptions?.beforeBuild || [])];

    beforeBuild.push(...(await this.fontProviderBeforeBuild()));

    if (this.accessModel != null || this.packageProvider != null) {
      const init = await import('@myriaddreamin/typst.ts/options.init');
      if (this.accessModel != null) {
        beforeBuild.push(init.withAccessModel(this.accessModel));
      }
      if (this.packageProvider != null) {
        beforeBuild.push(init.withPackageRegistry(this.packageProvider));
      }
    }

    initOptions.beforeBuild = beforeBuild;
    return initOptions;
  }

  async fontProviderBeforeBuild() {
    if (this.fontProvider === undefined || this.fontProvider === null) {
      return [];
    }

    const provider = normalizeFontProvider(this.fontProvider);
    if (provider.beforeBuild != null) {
      return Array.isArray(provider.beforeBuild) ? provider.beforeBuild : [provider.beforeBuild];
    }

    const fonts = collectFontInputs(provider);
    const init = await import('@myriaddreamin/typst.ts/options.init');
    return [init.loadFonts(fonts, provider.loadOptions ?? provider.options)];
  }

  replayShadowEntries() {
    for (const [path, entry] of this.shadowEntries) {
      if (entry.kind === 'source') {
        this.inner.addSource(path, entry.content);
      } else {
        this.inner.mapShadow(path, entry.content);
      }
    }
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
    const result = this.inner.addSource(path, source);
    this.shadowEntries.set(path, { kind: 'source', content: source });
    return result;
  }

  mapShadow(path, content) {
    const bytes = toUint8Array(content);
    const result = this.inner.mapShadow(path, bytes);
    this.shadowEntries.set(path, { kind: 'shadow', content: new Uint8Array(bytes) });
    return result;
  }

  unmapShadow(path) {
    const result = this.inner.unmapShadow(path);
    this.shadowEntries.delete(path);
    return result;
  }

  resetShadow() {
    const result = this.inner.resetShadow();
    this.shadowEntries.clear();
    return result;
  }

  reset() {
    return this.inner.reset();
  }

  async setFontProvider(provider) {
    this.fontProvider = provider;
    await this.rebuild();
  }

  async setAccessModel(accessModel) {
    this.accessModel = accessModel;
    await this.rebuild();
  }

  async setPackageProvider(packageProvider) {
    this.packageProvider = packageProvider;
    await this.rebuild();
  }

  async setPackageRegistry(packageRegistry) {
    return this.setPackageProvider(packageRegistry);
  }

  async prepareInput(input) {
    const opts = normalizeInput(input);
    const mainFilePath = mainPathOf(opts);

    if (opts.mainFileContent != null) {
      this.addSource(mainFilePath, opts.mainFileContent);
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

function normalizeFontProvider(provider) {
  if (Array.isArray(provider)) {
    return { fonts: provider };
  }

  if (provider && typeof provider === 'object') {
    return provider;
  }

  throw new TypeError('font provider must be an array or object');
}

function collectFontInputs(provider) {
  const fonts = [];
  appendArray(fonts, provider.fonts);
  appendArray(fonts, provider.rawFonts);
  appendArray(fonts, provider.fontData);
  appendArray(fonts, provider.lazyFonts);

  if (fonts.length === 0) {
    throw new TypeError('font provider must include fonts, rawFonts, fontData, lazyFonts, or beforeBuild');
  }

  return fonts;
}

function appendArray(target, value) {
  if (value == null) {
    return;
  }

  if (Array.isArray(value)) {
    target.push(...value);
    return;
  }

  target.push(value);
}

function toUint8Array(value) {
  return value instanceof Uint8Array
    ? value
    : new Uint8Array(value.buffer || value);
}

function optionValue(options, nested, key) {
  if (Object.prototype.hasOwnProperty.call(nested, key)) {
    return nested[key];
  }
  return options[key];
}
