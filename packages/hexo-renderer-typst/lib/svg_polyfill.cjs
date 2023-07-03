/// https://segmentfault.com/a/1190000016574288
(function () {
  var ie = !!(window.attachEvent && !window.opera);
  var wk = /webkit\/(\d+)/i.test(navigator.userAgent) && RegExp.$1 < 525;
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

document.ready(() => {
  let fromCtrlC = false;
  let fromRightClick = false;
  document.addEventListener(
    'keydown',
    evt => (fromCtrlC = evt.key.toUpperCase() == 'C' && evt.ctrlKey),
  );
  document.addEventListener('contextmenu', evt => (fromRightClick = true));
  document.addEventListener('copy', evt => {
    const originatorEvent = fromCtrlC ? 'CTRL-C' : fromRightClick ? 'right click' : 'toolbar menu';
    console.log(`Copy event received from ${originatorEvent} within element`, evt.target);
    fromCtrlC = false;
    fromRightClick = false;

    // evt.clipboardData.setData('text/plain', 'foo');
    // evt.preventDefault(); // default behaviour is to copy any selected text
  });
});
