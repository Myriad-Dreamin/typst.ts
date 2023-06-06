const text_layer_css = `
.typst-app {
  margin: 0;
}

.text-layer {
  position: relative;
  left: 0;
  top: 0;
  right: 0;
  bottom: 0;
  overflow: hidden;
  opacity: 0.2;
  line-height: 1;
}

.text-layer > div {
  color: transparent;
  /* position: absolute; */
  white-space: pre;
  cursor: text;
  transform-origin: 0% 0%;
}
`;

const index_js = `
/// https://segmentfault.com/a/1190000016574288
(function () {
  var ie = !!(window.attachEvent && !window.opera);
  var wk = /webkit\\/(\\d+)/i.test(navigator.userAgent) && RegExp.$1 < 525;
  var fn = [];
  var run = function () {
    for (var i = 0; i < fn.length; i++) fn[i]();
  };
  var d = document;
  d.ready = function (f) {
    if (!ie && !wk && d.addEventListener) return d.addEventListener('DOMContentLoaded', f, false);
    if (fn.push(f) > 1) return;
    if (ie)
      (function () {
        try {
          d.documentElement.doScroll('left');
          run();
        } catch (err) {
          setTimeout(arguments.callee, 0);
        }
      })();
    else if (wk)
      var t = setInterval(function () {
        if (/^(loaded|complete)$/.test(d.readyState)) clearInterval(t), run();
      }, 0);
  };
})();
`;

module.exports = function (locals) {
  return [
    `<style>${text_layer_css}</style>`,
    `<script>${index_js}</script>`,
    `<script type="module" src="/typst/typst-main.js"></script>`,
  ].join('\n');
};
