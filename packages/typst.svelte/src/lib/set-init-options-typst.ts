// Prevents reinitialization of compiler and renderer options during HMR (Hot Module Replacement).
// Use prepareUseOnce flag ensures initialization occurs only once to avoid duplicate calls to setXXXInitOptions.
import { $typst } from '@myriaddreamin/typst.ts';
import compilerWasmUrl from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import rendererWasmUrl from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';
let inited = false;

export default () => {
  if (!inited) {
    $typst.setCompilerInitOptions({
      beforeBuild: [],
      getModule: () =>
        compilerWasmUrl,
    });

    $typst.setRendererInitOptions({
      beforeBuild: [],
      getModule: () => rendererWasmUrl,
    });
    inited = true;
  }
  return $typst
};
