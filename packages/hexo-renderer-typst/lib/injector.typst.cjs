const fs = require('fs');
const path = require('path');

const typst_css = fs.readFileSync(path.resolve(path.dirname(__filename), 'typst.css'), 'utf-8');

const svg_polyfill_js = fs.readFileSync(
  path.resolve(path.dirname(__filename), 'svg_polyfill.cjs'),
  'utf-8',
);

module.exports = function (locals) {
  return [
    `<style>${typst_css}</style>`,
    `<script>${svg_polyfill_js}</script>`,
    `<script type="module" src="/typst/typst-main.js"></script>`,
  ].join('\n');
};
