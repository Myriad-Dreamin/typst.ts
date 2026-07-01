import { BACKENDS, selectAutoBackend } from './shared.mjs';

export async function createCompiler(options = {}) {
  const requested = options.backend || BACKENDS.auto;
  const backend = requested === BACKENDS.auto ? selectAutoBackend() : requested;

  switch (backend) {
    case BACKENDS.node: {
      const { createNodeCompiler } = await import('./node.mjs');
      return createNodeCompiler(options);
    }
    case BACKENDS.wasm: {
      const { createWasmCompiler } = await import('./wasm.mjs');
      return createWasmCompiler(options);
    }
    case BACKENDS.cli: {
      const { createCliCompiler } = await import('./cli.mjs');
      return createCliCompiler(options);
    }
    default:
      throw new Error(`Unknown compiler backend: ${String(backend)}`);
  }
}

export { createNodeCompiler } from './node.mjs';
export { createWasmCompiler } from './wasm.mjs';
export { createCliCompiler } from './cli.mjs';
