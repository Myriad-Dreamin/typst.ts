// debounce https://stackoverflow.com/questions/23181243/throttling-a-mousemove-event-to-fire-no-more-than-5-times-a-second
// ignore fast events, good for capturing double click
// @param (callback): function to be run when done
// @param (delay): integer in milliseconds
// @param (id): string value of a unique event id
// @doc (event.timeStamp): http://api.jquery.com/event.timeStamp/
// @bug (event.currentTime): https://bugzilla.mozilla.org/show_bug.cgi?id=238041
const ignoredEvent = (function () {
  const last: Record<string, number> = {};
  let diff, time;

  return function (callback: any, delay: any, id: string) {
    time = new Date().getTime();
    id = id || 'ignored event';
    diff = last[id] ? time - last[id] : time;

    if (diff > delay) {
      last[id] = time;
      callback();
    }
  };
})();

/// Filter HTMLCollection by fn
const fc = <T extends Element = Element>(
  collection: HTMLCollection,
  fn: (elem: T) => boolean,
): T[] => {
  const res: T[] = [];
  for (let i = 0; i < collection.length; i++) {
    const elem = collection[i] as T;
    if (fn(elem)) {
      res.push(elem);
    }
  }
  return res;
};

/// Check whether two dom rects are overlapping
const overLappingDom = (a: DOMRect, b: Pick<DOMRect, 'left' | 'right' | 'top' | 'bottom'>) => {
  return !(a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom);
};

/// Check whether two elements are almost overlapping
const almostOverLapping = (a: Element, b: Element) => {
  const aRect = a.getBoundingClientRect();
  const bRect = b.getBoundingClientRect();

  return (
    overLappingDom(aRect, bRect) &&
    /// determine overlapping by area
    Math.abs(aRect.left - bRect.left) + Math.abs(aRect.right - bRect.right) <
      0.5 * Math.max(aRect.width, bRect.width) &&
    Math.abs(aRect.bottom - bRect.bottom) + Math.abs(aRect.top - bRect.top) <
      0.5 * Math.max(aRect.height, bRect.height)
  );
};

interface ElementState {
  target?: Element & { relatedElements?: Element[] };
}

const gr = (window.typstGetRelatedElements = (elem: Element & { relatedElements?: Element[] }) => {
  let relatedElements = elem.relatedElements;
  if (relatedElements === undefined || relatedElements === null) {
    relatedElements = elem.relatedElements = searchIntersections(elem);
  }
  return relatedElements;
});

/// Get all related elements of an event target (must be an element)
const getRelatedElements = (event: Event & ElementState) => gr(event.target);

const findAncestor = (el: Element, cls: string) => {
  while (el && !el.classList.contains(cls)) el = el.parentElement!;
  return el;
};

function findGlyphListForText(n: Element) {
  const textEl = findAncestor(n, 'typst-text')!;
  return textEl && fc(textEl.children, e => e.tagName === 'use');
}

const searchIntersections = function (root: Element) {
  const groupEl = findAncestor(root, 'typst-group');
  return groupEl && fc(groupEl.children, e => almostOverLapping(e, root));
};

function nextNode(node: Node) {
  if (node.hasChildNodes()) {
    return node.firstChild;
  } else {
    while (node && !node.nextSibling) {
      node = node.parentNode!;
    }
    if (!node) {
      return null;
    }
    return node.nextSibling;
  }
}

function getRangeSelectedNodes(range: Range, filter: (a: any) => boolean | undefined) {
  var node = range.startContainer;
  var endNode = range.endContainer;

  // Special case for a range that is contained within a single node
  if (node == endNode) {
    if (filter(node)) {
      return [node];
    }
    if (filter(node.parentElement)) {
      return [node.parentElement];
    }
  }

  // Iterate nodes until we hit the end container
  var rangeNodes = [];
  while (node && node != endNode) {
    node = nextNode(node)!;
    if (filter(node)) {
      rangeNodes.push(node);
    }
  }

  // Add partially selected nodes at the start of the range
  node = range.startContainer;
  while (node && node != range.commonAncestorContainer) {
    if (filter(node)) rangeNodes.unshift(node);
    node = node.parentNode!;
  }

  return rangeNodes;
}

function getSelectedNodes(filter: (a: any) => boolean | undefined) {
  if (window.getSelection) {
    var sel = window.getSelection()!;
    if (!sel.isCollapsed) {
      if (sel.rangeCount === 1) {
        return getRangeSelectedNodes(sel.getRangeAt(0), filter);
      }

      let result = [];
      for (let i = 0, e = sel.rangeCount; i < e; i++) {
        result.push(...getRangeSelectedNodes(sel.getRangeAt(i), filter));
      }

      return result;
    }
  }
  return [];
}

function getGlyphLenShape(glyphRefs: Element[]) {
  return glyphRefs.map(e => {
    const href = e.getAttribute('href')!;
    const e2 = document.getElementById(href.slice(1));
    return 1 + Number.parseInt(e2?.getAttribute('data-liga-len') || '0');
  });
}

function getGlyphAdvanceShape(glyphRefs: Element[]) {
  return glyphRefs.map(e => {
    return Number.parseInt(e.getAttribute('x')! || '0');
  });
}

function adjsutTextSelection(docRoot: Element, textFlowCache: TextFlowCache) {
  docRoot.addEventListener('copy', (event: ClipboardEvent) => {
    const selection = getSelectedNodes(
      (n: Element) =>
        n.classList?.contains('tsel') ||
        n.classList?.contains('tsel-tok') ||
        n.classList?.contains('typst-content-hint'),
    ) as Element[];

    const textContent = [];
    let prevContent = false;
    for (let n of selection) {
      if (n.classList.contains('tsel')) {
        if (!n.hasAttribute('data-typst-layout-checked')) {
          textContent.push(n.textContent);
        }
        prevContent = true;
      } else if (n.classList.contains('tsel-tok')) {
        textContent.push(n.textContent);
        // console.log(n, n.textContent);
      } else if (prevContent) {
        const hint =
          String.fromCodePoint(Number.parseInt(n.getAttribute('data-hint') || '0', 16)) || '\n';
        // console.log('hint', hint);
        textContent.push(hint);
        prevContent = true; // collapse lines if false;
      }
    }

    const text = textContent.join('').replace(/\u00a0/g, ' ');

    console.log('user copy', text);
    if (navigator?.clipboard) {
      // console.log('clipboard api', text);
      navigator.clipboard.writeText(text);
    } else {
      event.clipboardData!.setData('text/plain', text);
    }
    event.preventDefault();
  });

  const pickElem = (t: Node): Element | null =>
    t.nodeType === Node.TEXT_NODE ? t.parentElement : (t as Element);
  const pickTselElem = (t: Node) => {
    const elem = pickElem(t);
    return elem?.classList?.contains('tsel') ? elem : undefined;
  };
  const createSelBox = (b: Pick<DOMRect, 'left' | 'top' | 'width' | 'height'>) => {
    return `<div style="position: absolute; float: left; left: ${b.left + window.scrollX}px; top: ${
      b.top + window.scrollY
    }px; width: ${b.width}px; height: ${b.height}px; background-color: #7db9dea0;"></div>`;
  };
  const clearSelBox = (selBox: Element | null) => {
    if (selBox) {
      selBox.innerHTML = '';
    }
  };

  let mouseLeftIsDown = false;
  window.addEventListener('mousedown', (event: MouseEvent) => {
    if (event.button === 0) {
      mouseLeftIsDown = true;
    }
  });
  window.addEventListener('mouseup', (event: MouseEvent) => {
    if (event.button === 0) {
      mouseLeftIsDown = false;
    }
  });
  docRoot.addEventListener('mousemove', (event: MouseEvent) => {
    if (mouseLeftIsDown) {
      ignoredEvent(
        () => {
          selectTextFlow(event);
        },
        2,
        'doc-text-sel',
      );
    }
  });

  function selectTextFlow(e: MouseEvent) {
    if ((e.target as HTMLSpanElement)?.classList.contains('tsel-tok')) {
      return;
    }
    updateSelection(true, e);
  }

  document.addEventListener('selectionchange', () => updateSelection(false));
  function updateSelection(isTextFlow: false): void;
  function updateSelection(isTextFlow: true, event: MouseEvent): void;
  function updateSelection(isTextFlow: boolean, event?: MouseEvent) {
    // const p0 = performance.now();
    const selection = window.getSelection();

    let selBox = document.getElementById('tsel-sel-box');

    if (!selection?.rangeCount) {
      // clean up
      clearSelBox(selBox);
      return;
    }

    // const p1 = performance.now();
    const rngBeg = selection.getRangeAt(0);
    const rngEnd = selection.getRangeAt(selection.rangeCount - 1);
    if (!rngBeg || !rngEnd) {
      return;
    }

    // const p2 = performance.now();
    const isPageGuardSelected = (ca: SVGGElement | null) => {
      return (
        ca?.classList.contains('text-guard') ||
        ca?.classList.contains('typst-page') ||
        ca?.classList.contains('typst-search-hint')
      );
    };

    const stIsPageGuard = isPageGuardSelected(
      pickElem(rngBeg.startContainer) as SVGGElement | null,
    );
    const edIsPageGuard = isPageGuardSelected(pickElem(rngEnd.endContainer) as SVGGElement | null);
    if (stIsPageGuard || edIsPageGuard) {
      // console.log('page guard selected');
      if (stIsPageGuard && edIsPageGuard) {
        clearSelBox(selBox);
      }
      return;
    }

    // const p3 = performance.now();

    // clean up
    clearSelBox(selBox);

    // const p4 = performance.now();

    if (!selBox) {
      selBox = document.createElement('div');
      selBox.id = 'tsel-sel-box';
      selBox.style.zIndex = '100';
      selBox.style.position = 'absolute';
      selBox.style.pointerEvents = 'none';
      selBox.style.left = '0';
      selBox.style.top = '0';
      selBox.style.float = 'left';
      document.body.appendChild(selBox);
    }

    const start = pickTselElem(rngBeg.startContainer);
    const end = pickTselElem(rngEnd.endContainer);

    const selectedTextList = getSelectedNodes(
      (n: Element) =>
        n.classList?.contains('tsel') ||
        n.classList?.contains('typst-search-hint') ||
        n.classList?.contains('tsel-tok'),
    ) as Element[];

    const selRng = new Range();
    const pieces: string[] = [];
    const createSelGlyphs = (st: Element, ed: Element, span: number) => {
      // console.log(st, span);
      selRng.setStartBefore(st);
      selRng.setEndAfter(ed);
      // selRng.getBoundingClientRect()

      // const x = st.getBoundingClientRect();
      // const y = ed.getBoundingClientRect();
      // const parent = st.parentElement!.getBoundingClientRect();
      // const z = {
      //   left: Math.min(x.left, y.left),
      //   right: Math.max(x.right, y.right),
      //   top: Math.min(x.top, y.top, parent.top),
      //   bottom: Math.max(x.bottom, y.bottom, parent.bottom),
      //   width: 0,
      //   height: 0,
      // };

      // z.width = z.right - z.left;
      // z.height = z.bottom - z.top;
      pieces.push(createSelBox(selRng.getBoundingClientRect()));
    };

    const tselRanges = new Map<Element, [number, number]>();
    // console.log('firefox', selectedTextList);

    // const p5 = performance.now();

    for (let n of selectedTextList) {
      if (n.classList.contains('tsel-tok')) {
        const n2 = n.parentElement!;
        const nth = Array.from(n2.children).indexOf(n);
        // console.log('tsel-tok', n, nth);
        if (!tselRanges.has(n2)) {
          tselRanges.set(n2, [nth, nth]);
        } else {
          const [st, ed] = tselRanges.get(n2)!;
          tselRanges.set(n2, [Math.min(st, nth), Math.max(ed, nth)]);
        }
      } else if (n.classList.contains('tsel') && !n.hasAttribute('data-typst-layout-checked')) {
        const st = n === start ? rngBeg.startOffset : 0;
        const ed = n === end ? rngEnd.endOffset - 1 : -1;
        tselRanges.set(n, [st, ed]);
      }
    }

    // const p6 = performance.now();

    if (isTextFlow) {
      let rngSt = 1e11,
        rngEd = -1;
      for (const div of tselRanges.keys()) {
        const idxStr = div.getAttribute('data-selection-index');
        if (!idxStr) {
          continue;
        }
        const idx = Number.parseInt(idxStr);
        rngSt = Math.min(rngSt, idx);
        rngEd = Math.max(rngEd, idx);
      }

      if (rngEd !== -1) {
        // console.log('text-flow', event, tselRanges, textFlowCache, rngSt, rngEd);
        const cx = event!.clientX;
        const cy = event!.clientY;

        const flow = textFlowCache.flow;
        for (;;) {
          const lastTsel = flow[rngEd];
          const bbox = lastTsel.getBoundingClientRect();
          // console.log('check last', lastTsel, bbox, cx, cy);
          if (cx > bbox.right || cy > bbox.bottom) {
            tselRanges.set(lastTsel, [0, -1]);

            if (rngEd + 1 < flow.length) {
              rngEd += 1;
              const nextTsel = flow[rngEd];
              const nxBbox = nextTsel.getBoundingClientRect();
              // todo: avoid assuming horizontal ltr
              if (bbox.bottom > nxBbox.top && bbox.top < nxBbox.bottom) {
                // console.log('same line', lastTsel, nextTsel);
                continue;
              }
            }
          }
          break;
        }
      }
    }

    // const p7 = performance.now();
    // console.log('firefox', tselRanges);

    // console.log(tselRanges);

    for (let [n, [st, ed]] of tselRanges) {
      const glyphRefs = findGlyphListForText(n);
      if (!glyphRefs?.length) {
        // console.log('no glyphs found...');
        continue;
      }

      if (st === 0 && ed === -1) {
        // console.log('select all', stGlyph, edGlyph);
        createSelGlyphs(glyphRefs[0], glyphRefs[glyphRefs.length - 1], ed);
        continue;
      }

      const glyphLens = getGlyphLenShape(glyphRefs);

      const findPos = (l: number) => {
        let pos = 0;
        for (let i = 0; i < glyphLens.length; i++) {
          if (pos + glyphLens[i] > l) {
            return glyphRefs[i];
          }
          pos += glyphLens[i];
        }
      };

      let stGlyph = glyphRefs[0];
      if (st !== 0) {
        stGlyph = findPos(st) || stGlyph;
      }

      let edGlyph = glyphRefs[glyphRefs.length - 1];
      if (ed !== -1) {
        edGlyph = findPos(ed) || edGlyph;
      }

      // console.log('select', st, ed, stGlyph, edGlyph, glyphLens, glyphRefs);
      createSelGlyphs(stGlyph, edGlyph, ed);
    }

    // const p8 = performance.now();

    selBox!.innerHTML = pieces.join('');

    // const p9 = performance.now();

    // const dms = (s: number, t: number) => `${(t - s).toFixed(2)} ms`;
    // let arr = [p0, p1, p2, p3, p4, p5, p6, p7, p8, p9];
    // const perf = arr.slice(1).map((t, i) => dms(arr[i], t));
    // console.log(`updateSelection ${perf.join(' ')}`);
  }
}

function createPseudoText(cls: string) {
  const foreignObj = document.createElementNS('http://www.w3.org/2000/svg', 'foreignObject');
  foreignObj.setAttribute('width', '1');
  foreignObj.setAttribute('height', '1');
  foreignObj.setAttribute('x', '0');
  foreignObj.setAttribute('y', '0');

  const tsel = document.createElement('span');
  tsel.textContent = '&nbsp;';
  // tsel.style.fontSize = '2048px';
  tsel.style.width = tsel.style.height = '100%';
  tsel.style.textAlign = 'justify';
  tsel.style.opacity = '0';
  tsel.classList.add(cls);
  foreignObj.append(tsel);

  return foreignObj;
}

/// Process mouse move event on pseudo-link elements
const linkmove = (event: Event & ElementState) =>
  ignoredEvent(() => gr(event.target)?.forEach(e => e.classList.add('hover')), 200, 'mouse-move');

/// Process mouse leave event on pseudo-link elements
const linkleave = (event: Event & ElementState) =>
  gr(event.target)?.forEach(e => e.classList.remove('hover'));

interface ProcessOptions {
  layoutText?: boolean;
}

// background: transparent;

interface TextFlowCache {
  flow: HTMLDivElement[];
}

window.typstProcessSvg = function (docRoot: SVGElement, options?: ProcessOptions) {
  if (false) {
    console.log('typst-text', docRoot.getElementsByClassName('typst-text').length);
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    Promise.all(
      Array.from(docRoot.getElementsByClassName('typst-text')).map(async g => {
        console.log('typst-text');
        const glyphs = Array.from(g.children).filter(e => e.tagName === 'typst-glyph');
        if (glyphs.length === 0) {
          return;
        }
        const firstGlyph = glyphs[0];

        const width =
          Math.max(...glyphs.map(e => Number.parseFloat(e.getAttribute('x') || '0'))) + 2048;
        canvas.width = width / 32;
        canvas.height = 2048 / 32;
        ctx.clearRect(0, 0, width / 32, 2048 / 32);
        ctx.scale(1 / 32, 1 / 32);
        ctx.translate(0, 2048 - 1440);
        let prevX = 0;
        for (const glyph of glyphs) {
          const x = Number.parseFloat(glyph.getAttribute('x') || '0');
          const href = glyph.getAttribute('href')!;
          const e = document.getElementById(href.slice(1)) as Element | null;
          if (e) {
            // translate x
            ctx.translate(x - prevX, 0);
            prevX = x;
            if (e.tagName === 'path') {
              const path = new Path2D(e.getAttribute('d')!);
              ctx.fillStyle = 'black';
              ctx.fill(path);
            } else {
              ctx.drawImage(e as SVGImageElement, 0, 0);
            }
          }
        }

        const imageUrl = canvas.toDataURL();

        // const generatedFrozenSvg = [];
        // const generatedFrozenSvgDefs = [];
        // const generatedFrozenSvgUses = [];
        const svgBBox = g.getBoundingClientRect();

        // generatedFrozenSvg.push(
        //   `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 ${
        //     114.44 * 16 - 2048
        //   } ${width} ${2048}" width="${width}px" height="${2048}px">`,
        // );
        // for (const glyph of glyphs) {
        //   const x = Number.parseFloat(glyph.getAttribute('x') || '0');
        //   const href = glyph.getAttribute('href')!;
        //   const e = document.getElementById(href.slice(1));
        //   if (e) {
        //     generatedFrozenSvgDefs.push(e!.outerHTML);
        //   }
        //   generatedFrozenSvgUses.push(`<use href="#${href.slice(1)}" x="${x}" />`);
        // }
        // generatedFrozenSvg.push(
        //   '<defs>',
        //   ...generatedFrozenSvgDefs,
        //   '</defs>',
        //   ...generatedFrozenSvgUses,
        //   '</svg>',
        // );
        // const svgText = generatedFrozenSvg.join('');

        // out of window
        // if (
        //   (svgBBox.right < 0 || svgBBox.right > window.innerWidth) &&
        //   (svgBBox.bottom < 0 || svgBBox.bottom > window.innerHeight) &&
        //   (svgBBox.left < 0 || svgBBox.left > window.innerWidth) &&
        //   (svgBBox.top < 0 || svgBBox.top > window.innerHeight)
        // ) {
        //   firstGlyph.remove();
        //   return;
        // }
        // the above check is not accurate, we need to consider intersection
        // if (
        //   !overLappingSimp(svgBBox, {
        //     left: 0,
        //     top: 0,
        //     right: window.innerWidth,
        //     bottom: window.innerHeight,
        //   })
        // ) {
        //   for (const glyph of glyphs) {
        //     glyph.remove();
        //   }
        //   return;
        // }

        // console.log('typst-text', g, firstGlyph.parentElement);
        // console.log(svgBBox, window.innerWidth, window.innerHeight);
        // const svgBlob = new Blob([svgText], { type: 'image/svg+xml;charset=utf-8' });
        // const imageUrl = URL.createObjectURL(svgBlob);

        // const img = document.createElementNS('http://www.w3.org/2000/svg', 'image');
        // img.setAttribute('preserveAspectRatio', 'none');
        // img.setAttribute('href', imageUrl);
        // img.setAttribute('x', '0');
        // img.setAttribute('width', width.toString());
        // img.setAttribute('height', '2048');
        // firstGlyph.replaceWith(img);
        // g.prepend(img);

        // const img = document.createElementNS('http://www.w3.org/2000/svg', 'image');
        // img.setAttribute('preserveAspectRatio', 'none');
        // img.setAttribute('href', imageUrl);
        // img.setAttribute('x', '0');
        // img.setAttribute('width', width.toString());
        // img.setAttribute('height', '2048');
        // firstGlyph.replaceWith(img);
        // g.prepend(img);

        const div = document.createElement('div');
        div.style.width = '100%';
        div.style.height = '100%';
        div.style.background =
          'conic-gradient(from 0.5turn at 50% 25%, red, orange, yellow, green, blue)';
        let fill = `url(${imageUrl})`;
        div.style.maskImage = fill;
        // webkit
        // div.style.webkitMaskImage = fill;
        div.style.setProperty('mask-image', fill);
        div.style.setProperty('-webkit-mask-image', fill);
        // console.log(fill);
        // mask size
        div.style.setProperty('mask-size', '100% 100%');
        div.style.setProperty('-webkit-mask-size', '100% 100%');
        // const htmlImage = new Image();
        // htmlImage.src = imageUrl;
        // htmlImage.setAttribute('preserveAspectRatio', 'none');
        // htmlImage.setAttribute('width', '100%');
        // htmlImage.setAttribute('height', '100%');
        // document.body.prepend(htmlImage);
        const foreignObj = document.createElementNS('http://www.w3.org/2000/svg', 'foreignObject');
        foreignObj.setAttribute('width', width.toString());
        foreignObj.setAttribute('height', '2048');
        foreignObj.setAttribute('x', '0');
        foreignObj.setAttribute('y', '0');
        // foreignObj.append(htmlImage);
        foreignObj.append(div);
        g.prepend(foreignObj);

        // document.body.prepend(img.cloneNode(true));
        for (const glyph of glyphs) {
          glyph.remove();
        }
      }),
    );
  }

  let textFlowCache: TextFlowCache = {
    flow: [],
  };

  var elements = docRoot.getElementsByClassName('pseudo-link');

  for (var i = 0; i < elements.length; i++) {
    var elem = elements[i];
    elem.addEventListener('mousemove', linkmove);
    elem.addEventListener('mouseleave', linkleave);
  }

  const layoutText = options?.layoutText ?? true;

  if (layoutText) {
    setTimeout(() => {
      // add rule: .tsel monospace
      // todo: outline styles
      const style = document.createElement('style');
      style.innerHTML = `.tsel { font-family: monospace; text-align-last: left !important; -moz-text-size-adjust: none; -webkit-text-size-adjust: none; text-size-adjust: none; }
.tsel span { float: left !important; position: absolute !important; width: fit-content !important; top: 0 !important; }
.typst-search-hint { font-size: 2048px; color: transparent; width: 100%; height: 100%; }
.typst-search-hint { color: transparent; user-select: none; }
.typst-search-hint::-moz-selection { color: transparent; background: #00000001; }
.typst-search-hint::selection { color: transparent; background: #00000001; }
.tsel span::-moz-selection,
.tsel::-moz-selection {
  background: transparent !important;
}
.tsel span::selection,
.tsel::selection {
  background: transparent !important;
} `;
      document.getElementsByTagName('head')[0].appendChild(style);

      // add css variable, font scale
      const devicePixelRatio = window.devicePixelRatio || 1;
      docRoot.style.setProperty('--typst-font-scale', devicePixelRatio.toString());
      window.addEventListener('resize', () => {
        const devicePixelRatio = window.devicePixelRatio || 1;
        docRoot.style.setProperty('--typst-font-scale', devicePixelRatio.toString());
      });

      window.layoutText(docRoot, textFlowCache);
    }, 0);

    adjsutTextSelection(docRoot, textFlowCache);
  }

  docRoot.addEventListener('click', (event: MouseEvent) => {
    let elem: HTMLElement | null = event.target as HTMLElement;
    while (elem) {
      const span = elem.getAttribute('data-span');
      if (span) {
        console.log('source-span of this svg element', span);

        const docRoot = document.body || document.firstElementChild;
        const basePos = docRoot.getBoundingClientRect();

        const vw = window.innerWidth || 0;
        const left = event.clientX - basePos.left + 0.015 * vw;
        const top = event.clientY - basePos.top + 0.015 * vw;

        triggerRipple(
          docRoot,
          left,
          top,
          'typst-debug-react-ripple',
          'typst-debug-react-ripple-effect .4s linear',
        );
        return;
      }
      elem = elem.parentElement;
    }
  });

  if (layoutText) {
    docRoot.querySelectorAll('.typst-page').forEach((e: Element) => {
      e.prepend(createPseudoText('text-guard'));
    });
  }

  if (window.location.hash) {
    // console.log('hash', window.location.hash);

    // parse location.hash = `loc-${page}x${x.toFixed(2)}x${y.toFixed(2)}`;
    const hash = window.location.hash;
    const hashParts = hash.split('-');
    if (hashParts.length === 2 && hashParts[0] === '#loc') {
      const locParts = hashParts[1].split('x');
      if (locParts.length === 3) {
        const page = Number.parseInt(locParts[0]);
        const x = Number.parseFloat(locParts[1]);
        const y = Number.parseFloat(locParts[2]);
        window.handleTypstLocation(docRoot, page, x, y);
      }
    }
  }
};

window.layoutText = function (svg: Element, textFlowCache: TextFlowCache) {
  const allElements = Array.from(svg.querySelectorAll('.tsel')) as HTMLDivElement[];
  textFlowCache.flow = allElements;
  const layoutBegin = performance.now();
  const ctx = (
    document.createElementNS('http://www.w3.org/1999/xhtml', 'canvas') as HTMLCanvasElement
  ).getContext('2d')!;
  // 128 * 16 = 2048
  ctx.font = `128px sans-serif`;
  const enCharWidth = ctx.measureText('A').width;
  // console.log('width of single char', enCharWidth);

  const searchHints = [];

  const layoutRange = (tselSt: number, tselEd: number) => {
    const divs = allElements.slice(tselSt, tselEd);
    tselSt -= 1;
    for (let d of divs) {
      tselSt += 1;
      if (d.getAttribute('data-typst-layout-checked')) {
        continue;
      }
      d.setAttribute('data-selection-index', tselSt.toString());
      d.setAttribute('data-typst-layout-checked', '1');

      if (d.style.fontSize) {
        const foreignObj = d.parentElement!;
        const textContent = d.innerText;

        // put search hint before foreignObj
        const hint = foreignObj.cloneNode(true) as Element;
        const firstSpan = hint.firstElementChild!;
        if (firstSpan) {
          firstSpan.className = 'typst-search-hint';
        }
        foreignObj.parentElement!.insertBefore(hint, foreignObj);

        searchHints.push([d, textContent]);

        const glyphs = findGlyphListForText(d);
        if (!glyphs) {
          // console.log('no glyphs found...');
          continue;
        }
        const glyphLens = getGlyphLenShape(glyphs);
        const glyphAdvances = getGlyphAdvanceShape(glyphs).map(t => t / 16);
        // console.log(
        //   d,
        //   targetWidth,
        //   currentX,
        //   'estimated',
        //   enCharWidth,
        //   d.clientWidth,
        //   charLen * enCharWidth,
        //   glyphs,
        //   textContent.split(''),
        //   glyphAdvances,
        //   glyphLens,
        // );

        let failed = false;
        const charContainers: HTMLSpanElement[] = [];
        let j = 0,
          k = 0;
        for (let c of textContent) {
          // console.log('c', c, j, k, glyphAdvances);
          if (j >= glyphAdvances.length) {
            failed = true;
            break;
          }
          let advance = glyphAdvances[j];
          if (glyphLens[j] > 1) {
            // console.log(
            //   'multi glyph estimated',
            //   c,
            //   glyphLens[j],
            //   glyphs[j].getBoundingClientRect().width,
            // );
            // advance += (k * glyphs[j].getBoundingClientRect().width) / glyphLens[j];
            // use enCharWidth instead of glyph width for speed
            advance += k * enCharWidth;
          }
          k++;
          if (k >= glyphLens[j]) {
            j++;
            k = 0;
          }

          const span = document.createElement('span');
          span.textContent = c;
          span.classList.add('tsel-tok');
          span.style.left = `${advance}px`;
          charContainers.push(span);
        }

        if (failed) {
          continue;
        }

        d.innerHTML = '';
        d.append(...charContainers);
        // console.log(d);
      }
    }

    console.log(
      `layoutText ${allElements.length} elements used since ${performance.now() - layoutBegin} ms`,
    );
  };

  // chunk elements
  const chunkSize = 100;
  for (let i = 0; i < allElements.length; i += chunkSize) {
    const chunkBegin = i;
    setTimeout(() => {
      layoutRange(chunkBegin, chunkBegin + chunkSize);
    });
  }
};

interface HandleOptions {
  behavior: ScrollBehavior;
}
window.handleTypstLocation = function (
  elem: Element,
  page: number,
  x: number,
  y: number,
  options?: HandleOptions,
) {
  const behavior = options?.behavior || 'smooth';
  const assignHashLoc =
    window.assignSemaHash ||
    ((u: number, x: number, y: number) => {
      // todo: multiple documents
      location.hash = `loc-${u}x${x.toFixed(2)}x${y.toFixed(2)}`;
    });
  const docRoot = findAncestor(elem, 'typst-doc');
  if (!docRoot) {
    console.warn('no typst-doc found', elem);
    return;
  }
  const children = docRoot.children;
  let nthPage = 0;
  for (let i = 0; i < children.length; i++) {
    if (children[i].tagName === 'g') {
      nthPage++;
    }
    if (nthPage == page) {
      // evaluate window viewport 1vw
      const pw = window.innerWidth * 0.01;
      const ph = window.innerHeight * 0.01;

      const page = children[i] as SVGGElement;
      const dataWidth =
        Number.parseFloat(
          docRoot.getAttribute('data-width') || docRoot.getAttribute('width') || '0',
        ) || 0;
      const dataHeight =
        Number.parseFloat(
          docRoot.getAttribute('data-height') || docRoot.getAttribute('height') || '0',
        ) || 0;
      // console.log(page, vw, vh, x, y, dataWidth, dataHeight, docRoot);
      const svgRectBase = docRoot.getBoundingClientRect();
      const svgRect = {
        left: svgRectBase.left,
        top: svgRectBase.top,
        width: svgRectBase.width,
        height: svgRectBase.height,
      };
      const xOffsetInnerFix = 7 * pw;
      const yOffsetInnerFix = 38.2 * ph;

      const transform = page.transform.baseVal.consolidate()?.matrix;
      if (transform) {
        // console.log(transform.e, transform.f);
        svgRect.left += (transform.e / dataWidth) * svgRect.width;
        svgRect.top += (transform.f / dataHeight) * svgRect.height;
      }

      const windowRoot = document.body || document.firstElementChild;
      const basePos = windowRoot.getBoundingClientRect();

      const xOffset =
        svgRect.left - basePos.left + (x / dataWidth) * svgRect.width - xOffsetInnerFix;
      const yOffset =
        svgRect.top - basePos.top + (y / dataHeight) * svgRect.height - yOffsetInnerFix;
      const left = xOffset + xOffsetInnerFix;
      const top = yOffset + yOffsetInnerFix;

      window.scrollTo({ behavior, left: xOffset, top: yOffset });

      if (behavior !== 'instant') {
        triggerRipple(
          windowRoot,
          left,
          top,
          'typst-jump-ripple',
          'typst-jump-ripple-effect .4s linear',
        );
      }

      assignHashLoc(nthPage, x, y);
      return;
    }
  }
};

function triggerRipple(
  docRoot: Element,
  left: number,
  top: number,
  className: string,
  animation: string,
) {
  const ripple = document.createElement('div');

  ripple.className = className;
  ripple.style.left = `${left}px`;
  ripple.style.top = `${top}px`;

  docRoot.appendChild(ripple);

  ripple.style.animation = animation;
  ripple.onanimationend = () => {
    docRoot.removeChild(ripple);
  };
}

var scriptTag = document.currentScript;
if (scriptTag) {
  console.log('new svg util updated 37  ', performance.now());
  const docRoot = findAncestor(scriptTag, 'typst-doc');
  if (docRoot) {
    window.typstProcessSvg(docRoot);
  }
}
