/**
 * The reference of a WebAssembly module which is copied from the wasm-bindgen
 * @see https://github.com/rustwasm/wasm-bindgen/blob/2c622715c9e6602f7bb377828c72f7953b178ed7/crates/cli-support/src/js/mod.rs#L656
 *
 * Your most common use case will be to pass a URL to a wasm file here.
 * + `WebAssembly.Module` - An instantiated wasm module.
 * + `URL` - Remote url to a wasm file
 * + `BufferSource` - An ArrayBufferView or an ArrayBuffer
 */
export type WebAssemblyModuleRef = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

/**
 * @internal
 */
const once = <T>(fn: () => T) => {
  let called = false;
  let res: T;
  return () => {
    if (called) {
      return res;
    }
    called = true;
    return (res = fn());
  };
};

/** @internal copied from wasm-bindgen */
type WasmModuleInitParam = WebAssemblyModuleRef | Promise<WebAssemblyModuleRef> | undefined;

/** @internal */
export class LazyWasmModule {
  private wasmBin: WasmModuleInitParam;
  private initOnce: () => Promise<void>;

  constructor(initFn: (param: WasmModuleInitParam) => Promise<unknown>) {
    if (typeof initFn !== 'function') {
      throw new Error('initFn is not a function');
    }

    this.initOnce = once(async () => {
      await initFn(this.wasmBin);
    });
  }

  async init(module?: WebAssemblyModuleRef | Promise<WebAssemblyModuleRef>) {
    this.wasmBin = module;
    await this.initOnce();
  }
}
