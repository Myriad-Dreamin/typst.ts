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

hexo.extend.injector.register('head_end', require('./lib/injector.pdf.cjs'), 'default');
hexo.extend.injector.register('head_end', require('./lib/injector.typst.cjs'), 'default');
hexo.extend.renderer.register('typst', 'html', render);
hexo.extend.renderer.register('typ', 'html', render);

hexo.extend.filter.register('after_post_render', process);

hexo.extend.generator.register('typst_assets', function (locals) {
  var assetPathConfig = hexo.config.asset_path;
  const base_dir = hexo.base_dir;
  const typst_main_path = path.resolve(
    base_dir,
    'node_modules/@myriaddreamin/typst.ts/dist/main.js',
  );

  const renderer_path = path.resolve(
    base_dir,
    'node_modules/@myriaddreamin/typst-ts-renderer/typst_ts_renderer_bg.wasm',
  );
  console.log(assetPathConfig);
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
  ];
});
