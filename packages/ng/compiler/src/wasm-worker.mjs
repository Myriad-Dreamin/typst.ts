import { isNodeLike, peerError, unsupported } from './shared.mjs';

const defaultWorkerUrl = new URL('./wasm-worker-entry.mjs', import.meta.url);

export async function createWasmWorkerCompiler(options = {}) {
  const workerOptions = options.wasmWorker || options;
  const worker = await createWorker(workerOptions);
  const facade = new WasmWorkerCompilerFacade(worker);

  try {
    await facade.init({
      wasmOptions: await prepareWasmOptionsForWorker(options),
    });
  } catch (error) {
    await facade.terminate();
    throw error;
  }

  return facade;
}

export class WasmWorkerCompilerFacade {
  backend = 'wasm-worker';

  constructor(worker) {
    this.worker = worker;
    this.nextId = 0;
    this.pending = new Map();
    this.disposed = false;
    this.unlisten = [
      listenWorkerMessage(worker, message => this.handleMessage(message)),
      listenWorkerError(worker, error => this.rejectAll(error)),
    ].filter(Boolean);
  }

  init(options) {
    return this.call('init', [options]);
  }

  compile(input) {
    return this.vector(input);
  }

  vector(input) {
    return this.call('vector', [input]);
  }

  pdf(input) {
    return this.call('pdf', [input]);
  }

  plainSvg(input) {
    return this.call('plainSvg', [input]);
  }

  svg(input) {
    return this.call('svg', [input]);
  }

  html(input) {
    return this.call('html', [input]);
  }

  query(input, options) {
    return this.call('query', [input, options]);
  }

  addSource(path, source) {
    return this.call('addSource', [path, source]);
  }

  mapShadow(path, content) {
    return this.call('mapShadow', [path, content]);
  }

  unmapShadow(path) {
    return this.call('unmapShadow', [path]);
  }

  resetShadow() {
    return this.call('resetShadow', []);
  }

  reset() {
    return this.call('reset', []);
  }

  setFontProvider(provider) {
    assertStructuredCloneable('font provider', provider);
    return this.call('setFontProvider', [provider]);
  }

  setAccessModel() {
    unsupported('wasm-worker', 'setAccessModel');
  }

  setPackageProvider() {
    unsupported('wasm-worker', 'setPackageProvider');
  }

  setPackageRegistry() {
    unsupported('wasm-worker', 'setPackageRegistry');
  }

  async terminate() {
    if (this.disposed) {
      return;
    }

    this.disposed = true;
    for (const unlisten of this.unlisten) {
      unlisten();
    }
    this.unlisten = [];
    this.rejectAll(new Error('wasm-worker backend was terminated'));

    if (typeof this.worker.terminate === 'function') {
      await this.worker.terminate();
    } else if (typeof this.worker.close === 'function') {
      this.worker.close();
    }
  }

  call(method, args) {
    if (this.disposed) {
      return Promise.reject(new Error('wasm-worker backend was terminated'));
    }

    const id = ++this.nextId;
    const promise = new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
    });
    try {
      postWorkerMessage(this.worker, { id, method, args });
    } catch (error) {
      this.pending.delete(id);
      return Promise.reject(error);
    }
    return promise;
  }

  handleMessage(message) {
    if (!message || typeof message !== 'object' || !message.id) {
      return;
    }

    const pending = this.pending.get(message.id);
    if (!pending) {
      return;
    }

    this.pending.delete(message.id);
    if (message.error) {
      pending.reject(deserializeError(message.error));
    } else {
      pending.resolve(message.result);
    }
  }

  rejectAll(error) {
    const pending = [...this.pending.values()];
    this.pending.clear();
    for (const item of pending) {
      item.reject(error instanceof Error ? error : new Error(String(error)));
    }
  }
}

async function createWorker(options = {}) {
  if (options.worker) {
    return options.worker;
  }

  if (options.workerFactory) {
    return options.workerFactory();
  }

  const url = options.workerUrl || defaultWorkerUrl;
  if (typeof Worker !== 'undefined') {
    return new Worker(url, { type: 'module', ...(options.workerOptions || {}) });
  }

  if (isNodeLike()) {
    let workerThreads;
    try {
      workerThreads = await import('node:worker_threads');
    } catch (error) {
      throw peerError('wasm-worker', 'node:worker_threads', error);
    }
    return new workerThreads.Worker(url, options.workerOptions || {});
  }

  throw new Error('The wasm-worker backend requires a Worker implementation.');
}

async function prepareWasmOptionsForWorker(options) {
  const workerOptions = options.wasmWorker || {};
  const wasmOptions = {
    ...(options.wasm || {}),
    ...(workerOptions.wasm || {}),
  };
  if (!Object.prototype.hasOwnProperty.call(wasmOptions, 'fontProvider') && 'fontProvider' in options) {
    wasmOptions.fontProvider = options.fontProvider;
  }
  for (const key of ['accessModel', 'packageProvider', 'packageRegistry']) {
    if (Object.prototype.hasOwnProperty.call(wasmOptions, key) || key in options) {
      throw new TypeError(`wasm-worker ${key} must be created inside the worker`);
    }
  }
  assertStructuredCloneable('font provider', wasmOptions.fontProvider);
  const initOptions = wasmOptions.initOptions || options.initOptions;

  if (!initOptions) {
    return wasmOptions;
  }

  const preparedInitOptions = { ...initOptions };
  if (typeof preparedInitOptions.getWrapper === 'function') {
    throw new TypeError('wasm-worker initOptions.getWrapper must be created inside the worker');
  }
  if (preparedInitOptions.beforeBuild?.some?.(item => typeof item === 'function')) {
    throw new TypeError('wasm-worker initOptions.beforeBuild must be created inside the worker');
  }

  if (typeof preparedInitOptions.getModule === 'function') {
    preparedInitOptions.module = await preparedInitOptions.getModule();
    delete preparedInitOptions.getModule;
  }

  wasmOptions.initOptions = preparedInitOptions;
  return wasmOptions;
}

function listenWorkerMessage(worker, cb) {
  if (typeof worker.addEventListener === 'function') {
    const handler = event => cb(event.data);
    worker.addEventListener('message', handler);
    return () => worker.removeEventListener?.('message', handler);
  }

  if (typeof worker.on === 'function') {
    worker.on('message', cb);
    return () => worker.off?.('message', cb) || worker.removeListener?.('message', cb);
  }

  worker.onmessage = event => cb(event.data);
  return () => {
    worker.onmessage = null;
  };
}

function listenWorkerError(worker, cb) {
  if (typeof worker.addEventListener === 'function') {
    const handler = event => cb(event.error || new Error(event.message || 'worker error'));
    worker.addEventListener('error', handler);
    return () => worker.removeEventListener?.('error', handler);
  }

  if (typeof worker.on === 'function') {
    worker.on('error', cb);
    worker.on('exit', code => {
      if (code !== 0) {
        cb(new Error(`worker exited with code ${code}`));
      }
    });
    return undefined;
  }

  worker.onerror = cb;
  return () => {
    worker.onerror = null;
  };
}

function postWorkerMessage(worker, message, transfer) {
  worker.postMessage(message, transfer || []);
}

function assertStructuredCloneable(name, value) {
  if (containsFunction(value, new Set())) {
    throw new TypeError(`wasm-worker ${name} must be structured-cloneable`);
  }
}

function containsFunction(value, seen) {
  if (typeof value === 'function') {
    return true;
  }
  if (!value || typeof value !== 'object') {
    return false;
  }
  if (seen.has(value)) {
    return false;
  }
  seen.add(value);
  if (ArrayBuffer.isView(value) || value instanceof ArrayBuffer) {
    return false;
  }
  for (const item of Object.values(value)) {
    if (containsFunction(item, seen)) {
      return true;
    }
  }
  return false;
}

function deserializeError(serialized) {
  const error = new Error(serialized.message || String(serialized));
  error.name = serialized.name || 'Error';
  error.stack = serialized.stack;
  return error;
}

export { createWasmWorkerCompiler as createCompiler };
