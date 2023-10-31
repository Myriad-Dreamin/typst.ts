export * from './all-in-one-lite.mjs';
import { $typst, TypstSnippet } from './snippet.mjs';
// @ts-ignore
import typstCompilerData from '../../../compiler/pkg/typst_ts_web_compiler_bg.wasm';
// @ts-ignore
import typstRendererData from '../../../renderer/pkg/typst_ts_renderer_bg.wasm';

(window as any).$wasm$typst_compiler = typstCompilerData;
(window as any).$wasm$typst_renderer = typstRendererData;

$typst.setCompilerInitOptions({
  getModule: () => (window as any).$wasm$typst_compiler,
});
$typst.setRendererInitOptions({
  getModule: () => (window as any).$wasm$typst_renderer,
});
