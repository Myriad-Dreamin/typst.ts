'use strict';

const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');

async function spawnAsync(cmd, args) {
  return new Promise((resolve, _reject) => {
    const child = spawn(cmd, args);

    child.stdout.on('data', data => {
      console.log(`stdout: ${data}`);
    });

    child.stderr.on('data', data => {
      console.error(`stderr: ${data}`);
    });

    child.on('error', error => {
      console.error(`error: ${error.message}`);
      reject(error);
    });

    child.on('close', code => {
      console.log(`child process exited with code ${code}`);
      resolve();
    });
  });
}

class Renderer {
  constructor(hexo) {
    this.hexo = hexo;
    this.renderCli = 'typst-ts-cli';
  }

  async render(data, _options) {
    const base_dir = this.hexo.base_dir;

    const rawDataPath = path
      .relative(base_dir, data.path)
      .replace(/\.[^/.]+$/, '')
      .replace(/\\/g, '/');
    const relDataPath = `artifacts/typst/${rawDataPath}`;
    const renderer_module = '/typst/typst_ts_renderer_bg.wasm';
    const dataPath = path.resolve(base_dir, 'public/', relDataPath);
    const dataDir = path.dirname(dataPath);
    console.log('dataPath', dataPath);
    fs.mkdirSync(dataDir, { recursive: true });

    await spawnAsync(this.renderCli, [
      'compile',
      '--workspace',
      base_dir,
      '--entry',
      data.path,
      '--output',
      dataDir,
      '--format=ir',
      '--format=json',
      '--format=json_glyphs',
    ]);

    await spawnAsync(this.renderCli, [
      'compile',
      '--workspace',
      base_dir,
      '--entry',
      data.path,
      '--output',
      dataDir,
      '--dynamic-layout',
    ]);

    const compiled = `
      <div id="typst-app"></div>
      <script>
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
              '${relDataPath}.multi.sir.bin',
            )
              .then(response => response.arrayBuffer())
              .then(buffer => new Uint8Array(buffer));

            const t0 = performance.now();

            const svgModule = await plugin.createModule(artifactData);
            let t1 = performance.now();

            console.log(\`init took \${t1 - t0} milliseconds\`);

            const appContainer = document.getElementById('typst-app');

            const runRender = async () => {
              t1 = performance.now();
              await plugin.renderSvg(svgModule, appContainer);

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
