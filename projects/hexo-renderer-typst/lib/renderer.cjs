'use strict';

const fs = require('fs');
const path = require('path');

class Renderer {
  constructor(hexo, compiler) {
    this.hexo = hexo;
    this.compiler = compiler;
  }

  async render(data, _options) {
    const base_dir = this.hexo.base_dir;

    const rawDataPath = path
      .relative(base_dir, data.path)
      .replace(/\.[^/.]+$/, '.multi.sir.in')
      .replace(/\\/g, '/');
    const relDataPath = `artifacts/typst/${rawDataPath}`;
    const renderer_module = '/typst/typst_ts_renderer_bg.wasm';
    const dataPath = path.resolve(base_dir, 'public/', relDataPath);
    const dataDir = path.dirname(dataPath);

    console.log('[typst] rendering', data.path, '...');
    const buf = this.compiler.vector(data.path);
    fs.mkdirSync(dataDir, { recursive: true });
    fs.writeFileSync(dataPath, buf);
    
    console.log('[typst] render   ', data.path, 'ok');

    const compiled = `
      <script>
        let appContainer = document.currentScript && document.currentScript.parentElement;
        document.ready(() => {
          let plugin = window.TypstRenderModule.createTypstSvgRenderer();
        console.log(plugin);
        plugin
          .init({
            getModule: () =>
              '${renderer_module}',
          })
          .then(async () => {
            const artifactData = await fetch(
              '/${relDataPath}',
            )
              .then(response => response.arrayBuffer())
              .then(buffer => new Uint8Array(buffer));

            const t0 = performance.now();

            const svgModule = await plugin.createModule(artifactData);
            let t1 = performance.now();

            console.log(\`init took \${t1 - t0} milliseconds\`);
            const appElem = document.createElement('div');
            appElem.class = 'typst-app';
            if (appContainer) {
              appContainer.appendChild(appElem);
            }

            const runRender = async () => {
              t1 = performance.now();
              await plugin.renderToSvg({ renderSession: svgModule, container: appElem });

              const t2 = performance.now();
              console.log(
                \`render took \${t2 - t1} milliseconds. total took \${t2 - t0} milliseconds.\`,
              );
            };

            let base = runRender();

            window.onresize = () => {
              base = base.then(runRender());
            };
          });
        });
      </script>`;
    return compiled;
  }
}

module.exports = Renderer;
