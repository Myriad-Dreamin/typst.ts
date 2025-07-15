import { InitOptions } from './options.init.mjs';

export async function createTypstRendererWorker() {
  const tsWrapper = 'http://127.0.0.1:20810/core/dist/esm/main.bundle.js';
  const rendererWrapper =
    'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer.mjs';
  const rendererWasm =
    'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm';

  let workerScript = `let renderer = null; let blobIdx = 0; let blobs = new Map();
function recvMsgOrLoadSvg({data}) { 
    if (data[0] && data[0].blobIdx) { console.log(data); let blobResolve = blobs.get(data[0].blobIdx); if (blobResolve) { blobResolve(data[1]); } return; }
    renderer.then(r => r.send(data)); }
self.loadSvg = function (data, format, w, h) { return new Promise(resolve => {
    blobIdx += 1; blobs.set(blobIdx, resolve); postMessage({ exception: 'loadSvg', token: { blobIdx }, data, format, w, h }, { transfer: [ data.buffer ] });
}); }

onmessage = recvMsgOrLoadSvg; const m = import(${JSON.stringify(tsWrapper)}); const s = import(${JSON.stringify(rendererWrapper)}); const w = fetch(${JSON.stringify(rendererWasm)}).then(r => r.arrayBuffer());
renderer = m
    .then((m) => { const r = m.createTypstRenderer(); return r.init({ beforeBuild: [], getWrapper: () => s, getModule: () => w }).then(_ => r.workerBridge()); })`;
  console.log('workerScript', workerScript);
  const blob = new Blob([workerScript], { type: 'application/javascript' });
  const worker = new Worker(URL.createObjectURL(blob), { type: 'module' });
  worker.addEventListener('message', e => {
    if (e.data.exception) {
      console.error('Worker exception:', e.data.exception);
    } else {
      console.log('Worker message:', e.data);
    }
  });
  worker.addEventListener('error', e => {
    console.error('Worker error:', e);
  });
  worker.addEventListener('messageerror', e => {
    console.error('Worker message error:', e);
  });
  return worker;
}

class TypstRendererWorker {
  constructor() {
    console.log('TypstRendererWorker created');
  }

  async init(options?: Partial<InitOptions>) {}
}
