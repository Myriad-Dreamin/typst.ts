export const BACKENDS = Object.freeze({
  auto: 'auto',
  wasm: 'wasm',
  wasmWorker: 'wasm-worker',
  node: 'node',
  cli: 'cli',
});

export function isNodeLike() {
  return (
    typeof process !== 'undefined' &&
    process.versions != null &&
    process.versions.node != null
  );
}

export function selectAutoBackend() {
  return isNodeLike() ? BACKENDS.node : BACKENDS.wasm;
}

export function normalizeInput(input) {
  if (!input || typeof input !== 'object') {
    throw new TypeError('compile input must be an object');
  }

  if (input.mainFileContent != null && typeof input.mainFileContent !== 'string') {
    throw new TypeError('mainFileContent must be a string');
  }

  if (input.mainContent != null && input.mainFileContent == null) {
    return { ...input, mainFileContent: input.mainContent };
  }

  return input;
}

export function mainPathOf(input) {
  return input.mainFilePath || '/main.typ';
}

export function peerError(backend, packageName, cause) {
  const error = new Error(
    `The ${backend} backend requires peer package "${packageName}". ` +
      `Install it next to @myriaddreamin/reflexo-typst-compiler or choose another backend.`,
  );
  error.cause = cause;
  return error;
}

export function unsupported(backend, method) {
  throw new Error(`The ${backend} backend does not support ${method}().`);
}
