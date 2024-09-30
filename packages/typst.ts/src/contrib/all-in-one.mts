export * from './all-in-one-lite.mjs';
import { $typst } from './snippet.mjs';
import typstCompilerData from '../../../compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import typstRendererData from '../../../renderer/pkg/typst_ts_renderer_bg.wasm?url';

(window as any).$wasm$typst_compiler = typstCompilerData;
(window as any).$wasm$typst_renderer = typstRendererData;

$typst.setCompilerInitOptions({
  getModule: () => (window as any).$wasm$typst_compiler,
});
$typst.setRendererInitOptions({
  getModule: () => (window as any).$wasm$typst_renderer,
});
