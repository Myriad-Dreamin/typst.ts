'use strict';

const fs = require('fs');
const path = require('path');

class Renderer {
  constructor(hexo) {
    this.hexo = hexo;
    this.renderCli = 'typst-ts-cli';
  }

  render(data, _options) {
    const base_dir = this.hexo.base_dir;

    const rawDataPath = path
      .relative(base_dir, data.path)
      .replace(/\.[^/.]+$/, '')
      .replace(/\\/g, '/');
    const relDataPath = `artifacts/typst/${rawDataPath}`;
    const dataPath = path.resolve(base_dir, 'public/', relDataPath);
    const dataDir = path.dirname(dataPath);
    console.log('dataPath', dataPath);
    fs.mkdirSync(dataDir, { recursive: true });

    return new Promise((resolve, _reject) => {
      const { spawn } = require('child_process');
      const args = [
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
      ];
      const child = spawn(this.renderCli, args);

      child.stdout.on('data', data => {
        console.log(`stdout: ${data}`);
      });

      child.stderr.on('data', data => {
        console.error(`stderr: ${data}`);
      });

      child.on('error', error => {
        console.error(`error: ${error.message}`);
      });

      const renderer_module = '/typst/typst_ts_renderer_bg.wasm';

      child.on('close', code => {
        console.log(`child process exited with code ${code}`);

        const compiled = `
        <div id="typst-app"></div>
        <script>
          document.ready(() => {
            let plugin = window.TypstRenderModule.createTypstRenderer(pdfjsLib);
            plugin
              .init({
                beforeBuild: [
                ],
                getModule: () => '${renderer_module}',
              })
              .then(() => {
                let artifactContent = undefined;
                const getGlyphs = fetch('/${relDataPath}.glyphs.json')
                  .then(response => response.text())
                  .then(content => JSON.parse(content))
                  .catch(e => {
                    console.log(e);
                    return undefined;
                  });
                fetch('/${relDataPath}.artifact.json')
                  .then(response => response.arrayBuffer())
                  .then(buffer => new Uint8Array(buffer))
                  .then(buffer => {
                    artifactContent = buffer;
                  })
                  .then(() => getGlyphs)
                  .then(glyphPack => plugin.loadGlyphPack(glyphPack))
                  .then(() => {
                    const appContainer = document.getElementById('typst-app');
                    plugin
                      .render({
                        artifactContent,
                        container: appContainer,
                        backgroundColor: '#ffffff',
                        pixelPerPt: 4.5,
                      })
                      .then(renderResult => {
                        console.log('render done');
                      });
                  });
              });
          });
        </script>`;
        resolve(compiled);
      });
    });
  }
}

module.exports = Renderer;
