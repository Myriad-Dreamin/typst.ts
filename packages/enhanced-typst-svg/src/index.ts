// debounce https://stackoverflow.com/questions/23181243/throttling-a-mousemove-event-to-fire-no-more-than-5-times-a-second
// ignore fast events, good for capturing double click
// @param (callback): function to be run when done
// @param (delay): integer in milliseconds
// @param (id): string value of a unique event id
// @doc (event.timeStamp): http://api.jquery.com/event.timeStamp/
// @bug (event.currentTime): https://bugzilla.mozilla.org/show_bug.cgi?id=238041
const ignoredEvent = (function () {
  var last: Record<string, number> = {},
    diff,
    time;

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

const overLappingSimp = function (a: Element, b: Element) {
  var aRect = a.getBoundingClientRect();
  var bRect = b.getBoundingClientRect();

  return !(
    aRect.right < bRect.left ||
    aRect.left > bRect.right ||
    aRect.bottom < bRect.top ||
    aRect.top > bRect.bottom
  );
};

const overLapping = function (a: Element, b: Element) {
  var aRect = a.getBoundingClientRect();
  var bRect = b.getBoundingClientRect();

  return (
    overLappingSimp(a, b) &&
    /// determine overlapping by area
    (Math.abs(aRect.left - bRect.left) + Math.abs(aRect.right - bRect.right)) /
      Math.max(aRect.width, bRect.width) <
      0.5 &&
    (Math.abs(aRect.bottom - bRect.bottom) + Math.abs(aRect.top - bRect.top)) /
      Math.max(aRect.height, bRect.height) <
      0.5
  );
};

var searchIntersections = function (root: Element) {
  let parent = undefined,
    current: Element | null = root;
  while (current) {
    if (current.classList.contains('typst-group')) {
      parent = current;
      break;
    }
    current = current.parentElement;
  }
  if (!current) {
    console.log('no group found');
    return;
  }
  const group = parent!;
  const children = group.children;
  const childCount = children.length;

  const res = [];

  for (let i = 0; i < childCount; i++) {
    const child = children[i];
    if (!overLapping(child, root)) {
      continue;
    }
    res.push(child);
  }

  return res;
};

interface ElementState {
  target?: Element & { relatedElements?: Element[] };
}

var getRelatedElements = function (event: Event & ElementState) {
  let relatedElements = event.target.relatedElements;
  if (relatedElements === undefined || relatedElements === null) {
    relatedElements = event.target.relatedElements = searchIntersections(event.target);
  }
  return relatedElements;
};

var linkmove = function (event: Event & ElementState) {
  ignoredEvent(
    function () {
      const elements = getRelatedElements(event);
      if (elements === undefined || elements === null) {
        return;
      }
      for (var i = 0; i < elements.length; i++) {
        var elem = elements[i];
        if (elem.classList.contains('hover')) {
          continue;
        }
        elem.classList.add('hover');
      }
    },
    200,
    'mouse-move',
  );
};

var linkleave = function (event: Event & ElementState) {
  const elements = getRelatedElements(event);
  if (elements === undefined || elements === null) {
    return;
  }
  for (var i = 0; i < elements.length; i++) {
    var elem = elements[i];
    if (!elem.classList.contains('hover')) {
      continue;
    }
    elem.classList.remove('hover');
  }
};

function findAncestor(el: Element, cls: string) {
  while (el && !el.classList.contains(cls)) el = el.parentElement!;
  return el;
}

function findGlyphListForText(n: Element) {
  const np = findAncestor(n, 'typst-text')!;
  if (!np) {
    // console.log('no typst-text found...');
    return undefined;
  }
  return Array.from(np.children).filter(e => e.tagName === 'use');
}

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
      return getRangeSelectedNodes(sel.getRangeAt(0), filter);
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

function adjsutTextSelection(docRoot: Element) {
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

    // console.log('user copy', text);
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
  const createSelBox = (selBox: Element, range: Range) => {
    const div = document.createElement('div');
    const b = range.getBoundingClientRect();
    div.style.position = 'absolute';
    div.style.float = 'left';
    div.style.left = `${b.left + window.scrollX}px`;
    div.style.top = `${b.top + window.scrollY}px`;
    div.style.width = `${b.width}px`;
    div.style.height = `${b.height}px`;
    div.style.backgroundColor = '#7db9dea0';
    selBox.appendChild(div);
  };
  const clearSelBox = (selBox: Element | null) => {
    if (selBox) {
      selBox.innerHTML = '';
    }
  };

  document.addEventListener('selectionchange', event => {
    const selection = window.getSelection();

    let selBox = document.getElementById('tsel-sel-box');

    if (!selection?.rangeCount) {
      // clean up
      clearSelBox(selBox);
      return;
    }

    const rng = selection?.getRangeAt(0);
    if (!rng) {
      return;
    }

    const isPageGuardSelected = (ca: SVGGElement | null) => {
      return (
        ca?.classList.contains('text-guard') ||
        ca?.classList.contains('typst-page') ||
        ca?.classList.contains('typst-search-hint')
      );
    };

    const stIsPageGuard = isPageGuardSelected(pickElem(rng.startContainer) as SVGGElement | null);
    const edIsPageGuard = isPageGuardSelected(pickElem(rng.endContainer) as SVGGElement | null);
    if (stIsPageGuard || edIsPageGuard) {
      console.log('page guard selected');
      if (stIsPageGuard && edIsPageGuard) {
        clearSelBox(selBox);
      }
      return;
    }

    // clean up
    clearSelBox(selBox);

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

    const start = pickTselElem(rng.startContainer);
    const end = pickTselElem(rng.endContainer);

    const selectedTextList = getSelectedNodes(
      (n: Element) =>
        n.classList?.contains('tsel') ||
        n.classList?.contains('typst-search-hint') ||
        n.classList?.contains('tsel-tok'),
    ) as Element[];

    const selRng = new Range();
    const createSelGlyphs = (st: Node, ed: Node) => {
      selRng.setStartBefore(st);
      selRng.setEndAfter(ed);
      createSelBox(selBox!, selRng);
    };

    const tselRanges = new Map<Element, [number, number]>();

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
        const st = n === start ? rng.startOffset : 0;
        const ed = n === end ? rng.endOffset - 1 : -1;
        tselRanges.set(n, [st, ed]);
      }
    }

    // console.log(tselRanges);

    for (let [n, [st, ed]] of tselRanges) {
      const glyphRefs = findGlyphListForText(n);
      if (!glyphRefs?.length) {
        // console.log('no glyphs found...');
        continue;
      }

      if (st === 0 && ed === -1) {
        // console.log('select all', stGlyph, edGlyph);
        createSelGlyphs(glyphRefs[0], glyphRefs[glyphRefs.length - 1]);
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
      createSelGlyphs(stGlyph, edGlyph);
    }
  });
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

interface ProcessOptions {
  layoutText?: boolean;
}

// background: transparent;

window.typstProcessSvg = function (docRoot: SVGElement, options?: ProcessOptions) {
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
.typst-search-hint { color: transpaent; user-select: none; }
.typst-search-hint::-moz-selection { color: transpaent; background: #00000001; }
.typst-search-hint::selection { color: transpaent; background: #00000001; }
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

      window.layoutText(docRoot);
    }, 0);

    adjsutTextSelection(docRoot);
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

window.layoutText = function (svg: Element) {
  const allElements = Array.from(svg.querySelectorAll('.tsel')) as HTMLDivElement[];
  const layoutBegin = performance.now();
  const ctx = (
    document.createElementNS('http://www.w3.org/1999/xhtml', 'canvas') as HTMLCanvasElement
  ).getContext('2d')!;
  // 128 * 16 = 2048
  ctx.font = `128px sans-serif`;
  const enCharWidth = ctx.measureText('A').width;
  // console.log('width of single char', enCharWidth);

  const searchHints = [];

  const layoutRange = (divs: HTMLDivElement[]) => {
    for (let d of divs) {
      if (d.getAttribute('data-typst-layout-checked')) {
        continue;
      }

      if (d.style.fontSize) {
        // scale:
        // const fontSize = Number.parseFloat(d.style.fontSize.replace('px', ''));
        // d.style.fontSize = `${fontSize}px`;

        // d.style.fontSize = 'calc(' + d.style.fontSize + ' * var(--typst-font-scale))';
        const foreignObj = d.parentElement!;
        // const innerText = d.innerText;
        // const targetWidth = Number.parseFloat(foreignObj.getAttribute('width')!);
        // const currentX = Number.parseFloat(foreignObj.getAttribute('x') || '0');
        // const currentY = Number.parseFloat(foreignObj.getAttribute('y') || '0');
        const textContent = d.innerText;

        // foreignObj
        // put search hint before foreignObj
        const hint = foreignObj.cloneNode(true) as Element;
        // hint.innerHTML = '<span class="typst-search-hint">' + textContent + '</span>';
        const firstSpan = hint.firstElementChild!;
        if (firstSpan) {
          firstSpan.className = 'typst-search-hint';
        }
        foreignObj.parentElement!.insertBefore(hint, foreignObj);

        // hint.setAttribute('width', '1');
        // hint.setAttribute('height', '1');
        // hint.setAttribute('x', foreignObj.getAttribute('x'));
        // hint.setAttribute('y', foreignObj.getAttribute('y'));

        searchHints.push([d, textContent]);

        const charLen = textContent.length;

        // hint.setAttribute(
        //   'width',
        //   (Number.parseFloat(foreignObj.getAttribute('width')!) / charLen).toString(),
        // );

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

        // split chars by glyphLens
        // const chars: string[] = [];
        // let splitMe = textContent;
        // for (let i = 0; i < glyphLens.length; i++) {
        //   const len = glyphLens[i];
        //   if (splitMe.length < len) {
        //     splitMe = undefined!;
        //     break;
        //   }
        //   const char = splitMe.slice(0, len);
        //   chars.push(char);
        //   splitMe = splitMe.slice(len);
        // }
        // if (splitMe === undefined || splitMe.length > 0) {
        //   console.log('split failed', d, textContent, glyphLens);
        //   continue;
        // }

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

        // const scale = targetWidth / selfWidth;
        // d.style.transform = `scaleX(${scale})`;
        // foreignObj.setAttribute('width', selfWidth);
        // foreignObj.setAttribute('x', currentX - (selfWidth - targetWidth) * 0.5);

        // const currentY = Number.parseFloat(foreignObj.getAttribute('y')!) || 0;
        // foreignObj.setAttribute('y', (currentY - 2500 / 16).toString());

        d.setAttribute('data-typst-layout-checked', '1');
        // return;
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
      layoutRange(allElements.slice(chunkBegin, chunkBegin + chunkSize));
    });
  }
};

window.handleTypstLocation = function (elem: Element, page: number, x: number, y: number) {
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
      const dataWidth = Number.parseFloat(docRoot.getAttribute('data-width') || '0') || 0;
      const dataHeight = Number.parseFloat(docRoot.getAttribute('data-height') || '0') || 0;
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

      const pageRect = page.getBoundingClientRect();

      const windowRoot = document.body || document.firstElementChild;
      const basePos = windowRoot.getBoundingClientRect();

      const xOffset =
        svgRect.left - basePos.left + (x / dataWidth) * svgRect.width - xOffsetInnerFix;
      const yOffset =
        svgRect.top - basePos.top + (y / dataHeight) * svgRect.height - yOffsetInnerFix;
      const left = xOffset + xOffsetInnerFix;
      const top = yOffset + yOffsetInnerFix;

      const widthOccupied = (100 * 100 * pw) / pageRect.width;

      const pageAdjustLeft = pageRect.left - basePos.left - 5 * pw;
      const pageAdjust = pageRect.left - basePos.left + pageRect.width - 95 * pw;

      // default single-column or multi-column layout
      if (widthOccupied >= 90 || widthOccupied < 50) {
        window.scrollTo({ behavior: 'smooth', left: xOffset, top: yOffset });
      } else {
        // for double-column layout
        // console.log('occupied adjustment', widthOccupied, page);

        const xOffsetAdjsut = xOffset > pageAdjust ? pageAdjust : pageAdjustLeft;

        window.scrollTo({ behavior: 'smooth', left: xOffsetAdjsut, top: yOffset });
      }

      triggerRipple(
        windowRoot,
        left,
        top,
        'typst-jump-ripple',
        'typst-jump-ripple-effect .4s linear',
      );
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
  // console.log('new svg util updated 30  ');
  const docRoot = findAncestor(scriptTag, 'typst-doc');
  if (docRoot) {
    window.typstProcessSvg(docRoot);
  }
}
