export type WebAssemblyModuleRef = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

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
