console.log('hello world');
import App from './App.vue';
import { createApp } from 'vue';

import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';

$typst.setCompilerInitOptions({
  beforeBuild: [],
  getModule: () =>
    // 'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
    // For local development
    'http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
});

$typst.setRendererInitOptions({
  beforeBuild: [],
  getModule: () =>
    // 'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
    // For local development
    'http://localhost:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
});

const app = createApp(App);

app.mount('#app');
