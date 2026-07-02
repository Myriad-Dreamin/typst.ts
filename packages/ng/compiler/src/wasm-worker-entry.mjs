import { createWasmCompiler } from './wasm.mjs';

let compiler;
let port = globalThis;

try {
  const workerThreads = await import('node:worker_threads');
  if (!workerThreads.isMainThread && workerThreads.parentPort) {
    port = workerThreads.parentPort;
  }
} catch {
  // Browser workers do not provide node:worker_threads.
}

listen(message => {
  handleMessage(message).catch(error => {
    post({
      id: message?.id,
      error: serializeError(error),
    });
  });
});

async function handleMessage(message) {
  if (!message || typeof message !== 'object') {
    return;
  }

  const { id, method, args = [] } = message;
  if (method === 'init') {
    const [{ wasmOptions } = {}] = args;
    compiler = await createWasmCompiler(restoreWasmOptions(wasmOptions || {}));
    post({ id, result: undefined });
    return;
  }

  if (!compiler) {
    throw new Error('wasm-worker compiler is not initialized');
  }

  const target = compiler[method];
  if (typeof target !== 'function') {
    throw new Error(`Unknown wasm-worker compiler method: ${String(method)}`);
  }

  const result = await target.apply(compiler, args);
  post({ id, result }, transferOf(result));
}

function restoreWasmOptions(wasmOptions) {
  const options = { wasm: { ...wasmOptions } };
  if (wasmOptions.initOptions) {
    const initOptions = { ...wasmOptions.initOptions };
    if ('module' in initOptions) {
      const module = initOptions.module;
      delete initOptions.module;
      initOptions.getModule = () => module;
    }
    options.wasm.initOptions = initOptions;
  }
  return options;
}

function listen(cb) {
  if (typeof port.addEventListener === 'function') {
    port.addEventListener('message', event => cb(event.data));
    return;
  }

  if (typeof port.on === 'function') {
    port.on('message', cb);
    return;
  }

  port.onmessage = event => cb(event.data);
}

function post(message, transfer) {
  if (typeof port.postMessage === 'function') {
    port.postMessage(message, transfer || []);
    return;
  }

  globalThis.postMessage(message, transfer || []);
}

function transferOf(result) {
  if (!(result instanceof Uint8Array)) {
    return [];
  }

  return [result.buffer];
}

function serializeError(error) {
  return {
    name: error?.name || 'Error',
    message: error?.message || String(error),
    stack: error?.stack,
  };
}
