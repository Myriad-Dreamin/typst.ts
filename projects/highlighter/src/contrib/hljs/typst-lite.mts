import typstInit from '@myriaddreamin/typst-ts-parser/pkg/wasm-pack-shim.mjs';
import { hljsTypst, initHljs } from '../../hljs.mjs';

type ModuleSource = 'local' | 'jsdelivr';

/**
 * The reference of a WebAssembly module which is copied from the wasm-bindgen
 * @see https://github.com/rustwasm/wasm-bindgen/blob/2c622715c9e6602f7bb377828c72f7953b178ed7/crates/cli-support/src/js/mod.rs#L656
 *
 * Your most common use case will be to pass a URL to a wasm file here.
 * + `WebAssembly.Module` - An instantiated wasm module.
 * + `URL` - Remote url to a wasm file
 * + `BufferSource` - An ArrayBufferView or an ArrayBuffer
 */
type WebAssemblyModuleRef = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

/// Begin of Retrieve Wasm Modules from somewhere
/// We need a compiler module and a parser module
/// - `@myriaddreamin/typst-ts-parser`

// Bundle
// @ts-ignore
// import parser from '@myriaddreamin/typst-ts-parser/pkg/typst_ts_parser_bg.wasm?url';

let moduleSource: ModuleSource = (window.$typst$parserModuleSource || 'jsdelivr') as any;

let parserModule: WebAssemblyModuleRef;

switch (moduleSource) {
  default:
    if (typeof moduleSource !== 'string') {
      parserModule = moduleSource;
    } else {
      parserModule = fetch(moduleSource).catch(error => {
        console.warn('unknown module source for importing typst module', moduleSource, error);
      });
    }
  case null:
  case undefined:
  case 'jsdelivr':
    parserModule = fetch(
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-parser/pkg/typst_ts_parser_bg.wasm',
    );
    break;
  case 'local':
    parserModule = fetch(
      'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-parser/pkg/typst_ts_parser_bg.wasm',
    );
    break;
}

/// End of Retrieve Wasm Modules from somewhere

window.$typst$parserModule = typstInit(parserModule).then(() => initHljs());
window.hljsTypst = hljsTypst;
