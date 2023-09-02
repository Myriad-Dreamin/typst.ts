/* global hexo */

'use strict';

const path = require('path');
const fs = require('fs');

const Renderer = require('./lib/renderer.cjs');
const renderer = new Renderer(hexo);

const Processor = require('./lib/processor.cjs');
const processor = new Processor(hexo);

function render(data, options) {
  return renderer.render(data, options);
}

function process(data) {
  return processor.process(data);
}

render.disableNunjucks = true;

hexo.extend.injector.register('head_end', require('./lib/injector.typst.cjs'), 'default');
hexo.extend.injector.register('head_end', require('./lib/injector.svg.cjs'), 'default');
hexo.extend.renderer.register('typst', 'html', render);
hexo.extend.renderer.register('typ', 'html', render);

hexo.extend.filter.register('after_post_render', process);

hexo.extend.generator.register('typst_assets', function (locals) {
  const base_dir = hexo.base_dir;
  const typst_main_path = path.resolve(
    base_dir,
    'node_modules/@myriaddreamin/typst.ts/dist/esm/main.bundle.js',
  );

  const svg_utils_path = path.resolve(path.dirname(__filename), 'lib/svg_utils.cjs');

  const renderer_path = path.resolve(
    base_dir,
    'node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
  );
  return [
    {
      path: 'typst/typst-main.js',
      data: function () {
        return fs.createReadStream(typst_main_path);
      },
    },
    {
      path: 'typst/typst_ts_renderer_bg.wasm',
      data: function () {
        return fs.createReadStream(renderer_path);
      },
    },
    {
      path: 'typst/svg-utils.js',
      data: function () {
        return fs.createReadStream(svg_utils_path);
      },
    },
  ];
});
