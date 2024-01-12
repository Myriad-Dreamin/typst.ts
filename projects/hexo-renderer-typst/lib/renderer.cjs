'use strict';

const fs = require('fs');
const path = require('path');

const trampoline_js = fs.readFileSync(
  path.resolve(path.dirname(__filename), 'trampoline.cjs'),
  'utf-8',
);

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

    const compiled = `<script>${trampoline_js}</script>`
      .replace('{{renderer_module}}', renderer_module)
      .replace('{{relDataPath}}', relDataPath);
    return compiled;
  }
}

module.exports = Renderer;
