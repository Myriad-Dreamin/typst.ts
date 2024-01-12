window.typstBindSemantics = function () {};
window.typstBindSvgDom = function () {};
window.captureStack = function () {
  return undefined;
};
let appContainer = document.currentScript && document.currentScript.parentElement;

const appElem = document.createElement('div');
if (appContainer) {
  appContainer.appendChild(appElem);
}
appElem.classList.add('typst-app');

function getViewport(appElem) {
  const domScale = 1.5;
  const appPos = appElem.getBoundingClientRect();
  const left = appPos.left;
  const top = -appPos.top;
  const right = window.innerWidth;
  const bottom = window.innerHeight - appPos.top;
  const rect = {
    x: 0,
    y: top / domScale,
    width: Math.max(right - left, 0) / domScale,
    height: Math.max(bottom - top, 0) / domScale,
  };
  if (rect.width <= 0 || rect.height <= 0) {
    rect.x = rect.y = rect.width = rect.height = 0;
  }
  // console.log('ccc', basePos, appPos, rect);
  return rect;
}

// prefetch
let tsModule = fetch(`{{renderer_module}}`);
let tsData = fetch(`/{{relDataPath}}`);

let prevHovers = void 0;
function updateHovers(elems) {
  if (prevHovers) {
    for (const h of prevHovers) {
      h.classList.remove("focus");
    }
  }
  prevHovers = elems;
}
let globalSemaLabels = [];

document.ready(() => {
  let plugin = window.TypstRenderModule.createTypstSvgRenderer();
  console.log(plugin);

  let isRendering = false;
  let renderResponsive = undefined;
  let session = undefined;
  let rerenderOverhead = 0;

  let initialRender = true;
  const typstBindCustomSemantics = async (root, svg, semantics) => {
    console.log('bind custom semantics', root, svg, semantics);
    const customs = await plugin.getCustomV1({
      renderSession: session,
    });
    const semaLabel = customs.find(k => k[0] === 'sema-label');
    if (semaLabel) {
      const labelBin = semaLabel[1];
      const labels = JSON.parse(dec.decode(labelBin));
      globalSemaLabels = labels.map(([label, pos]) => {
        const [_, u, x, y] = pos.split(/[pxy]/).map(Number.parseFloat);
        return [encodeURIComponent(label), svg, [u, x, y]];
      });
    }

    postProcessCrossLinks(semantics);

    // todo: out of page
    if (window.location.hash) {
      // console.log('hash', window.location.hash);

      // parse location.hash = `loc-${page}x${x.toFixed(2)}x${y.toFixed(2)}`;
      const hash = window.location.hash;
      const firstSep = hash.indexOf('-');
      // console.log('jump label', window.location.hash, firstSep, globalSemaLabels);
      if (firstSep != -1 && hash.slice(0, firstSep) === '#label') {
        const labelTarget = hash.slice(firstSep + 1);
        for (const [label, dom, pos] of globalSemaLabels) {
          if (label === labelTarget) {
            const [_, x, y] = pos;
            // console.log('jump label', label, pos);
            window.handleTypstLocation(dom, 1, x, y, {
              behavior: initialRender ? 'smooth' : 'instant',
            });
            initialRender = false;
            break;
          }
        }
      }
    }
  };

  window.typstBindCustomSemantics = (root, svg, semantics) =>
    setTimeout(() => typstBindCustomSemantics(root, svg, semantics), 0);

  const baseHandleTypstLocation = window.handleTypstLocation;
  window.handleTypstLocation = (elem, page, x, y, options) => {
    const docRoot = findAncestor(elem, 'typst-app');
    if (!docRoot) {
      console.warn('no typst-app found', elem);
      return;
    }

    console.log(docRoot);
    options = options || {};
    options.isDom = true;

    for (const h of docRoot.children) {
      if (h.classList.contains('typst-dom-page')) {
        const idx = Number.parseInt(h.getAttribute('data-index'));
        if (idx + 1 === page) {
          const svg = h.querySelector('.typst-svg-page');
          if (svg) {
            baseHandleTypstLocation(svg, page, x, y, options);
          }
          return;
        }
      }
    }
  };

  window.assignSemaHash = (u, x, y) => {
    // console.log(`find labels ${u}:${x}:${y} in`, globalSemaLabels);
    for (const [label, dom, pos] of globalSemaLabels) {
      const [u1, x1, y1] = pos;
      if (u === u1 && Math.abs(x - x1) < 0.01 && Math.abs(y - y1) < 0.01) {
        location.hash = `label-${label}`;
        // const domX1 = x1 * dom.viewBox.baseVal.width;
        // const domY1 = y1 * dom.viewBox.baseVal.height;

        window.typstCheckAndRerender?.(false, new Error('assignSemaHash')).then(() => {
          const width = dom.viewBox.baseVal.width;
          const height = dom.viewBox.baseVal.height;
          const bbox = dom.getBoundingClientRect();
          const domX1 = bbox.left + (x1 / width) * bbox.width;
          const domY1 = bbox.top + (y1 / height) * bbox.height;

          const lnk = findLinkInSvg(dom, [domX1, domY1]);
          if (!lnk) {
            return;
          }
          // const semaLinkLocation = document.getElementById(`typst-label-${label}`);
          const relatedElems = window.typstGetRelatedElements(lnk);
          for (const h of relatedElems) {
            h.classList.add('focus');
          }
          updateHovers(relatedElems);
          return;
        });
        return;
      }
    }
    updateHovers([]);
    // todo: multiple documents
    location.hash = `loc-${u}x${x.toFixed(2)}x${y.toFixed(2)}`;
  };

  let initialized = plugin.init({ getModule: () => tsModule });

  initialized = initialized
    .then(() => tsData)
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  initialized = initialized.then(artifactData => {
    return new Promise(resolve => {
      plugin.runWithSession(
        ses =>
          new Promise(dispose => {
            // ignore dispose
            void dispose;

            session = ses;
            plugin.manipulateData({ renderSession: ses, data: artifactData });

            const t = performance.now();
            const p = plugin.renderDom({
              renderSession: ses,
              container: appElem,
              pixelPerPt: 3,
              viewport: getViewport(appElem),
            });
            resolve(
              p.then(() => {
                const rTime = performance.now() - t;
                console.log('plugin.renderDom', performance.now() - t);
                rerenderOverhead = rTime + rerenderOverhead * 0.5;
                window.rerenderOverhead = rerenderOverhead;
              }),
            );
          }),
      );
    });
  });

  function queueRerender(stack) {
    if (renderResponsive === undefined) {
      if (stack) {
        console.log('skip', stack);
      }
      isRendering = false;
      return;
    }
    isRendering = true;
    let responsive = renderResponsive === false ? false : true;
    renderResponsive = undefined;
    const t = performance.now();
    console.log('rerender, overhead: ', rerenderOverhead);

    plugin
      .triggerDomRerender({
        renderSession: session,
        responsive,
        viewport: getViewport(appElem),
      })
      .then(() => {
        const rTime = performance.now() - t;
        if (!responsive) {
          rerenderOverhead = rTime + rerenderOverhead * 0.5;
          window.rerenderOverhead = rerenderOverhead;
        }
        if (stack) {
          console.log('pull render', rTime, responsive, stack);
        }

        return queueRerender();
      });
  }

  const checkAndRerender = (r, stack) => {
    if (r !== true && r !== false) {
      throw new Error('invalid responsive');
    }
    if (r === false) {
      renderResponsive = false;
    } else if (renderResponsive !== false) {
      renderResponsive = true;
    }

    if (stack) {
      console.log('submit', stack);
    }

    if (!isRendering) {
      queueRerender(stack);
    }
  };

  let responsiveTimeout = undefined;
  let responsiveTimeout2 = undefined;
  const responsiveAction = stack => {
    stack ||= window.captureStack();
    clearTimeout(responsiveTimeout);
    clearTimeout(responsiveTimeout2);
    console.log('responsiveAction', rerenderOverhead, stack);
    responsiveTimeout = setTimeout(
      () => {
        clearTimeout(responsiveTimeout);
        clearTimeout(responsiveTimeout2);
        checkAndRerender(true, stack);
        responsiveTimeout2 = setTimeout(
          () => checkAndRerender(false, stack),
          Math.max(200, rerenderOverhead * 2.5),
        );
      },
      Math.max(10, rerenderOverhead * 1.1),
    );
  };

  initialized.then(() => {
    window.addEventListener('resize', () => responsiveAction());
    window.addEventListener('scroll', () => responsiveAction());
  });
});
