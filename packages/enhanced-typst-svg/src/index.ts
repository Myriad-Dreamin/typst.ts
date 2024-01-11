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

interface DOMBox {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/// Check whether two dom rects are overlapping
const overLappingBox = (a: DOMBox, b: DOMBox) => {
  return !(a.right < b.left || a.left > b.right || a.bottom < b.top || a.top > b.bottom);
};

/// Check whether two elements are almost overlapping
const almostOverLapping = (a: Element, b: Element) => {
  const aRect = a.getBoundingClientRect();
  const bRect = b.getBoundingClientRect();

  return (
    overLappingBox(aRect, bRect) &&
    /// determine overlapping by area
    Math.abs(aRect.left - bRect.left) + Math.abs(aRect.right - bRect.right) <
      0.5 * Math.max(aRect.width, bRect.width) &&
    Math.abs(aRect.bottom - bRect.bottom) + Math.abs(aRect.top - bRect.top) <
      0.5 * Math.max(aRect.height, bRect.height)
  );
};

interface ElementState extends Element {
  relatedElements?: Element[];
}

const gr = (window.typstGetRelatedElements = (elem: ElementState) => {
  let relatedElements = elem.relatedElements;
  if (relatedElements === undefined || relatedElements === null) {
    relatedElements = elem.relatedElements = searchIntersections(elem);
  }
  return relatedElements;
});

/// Get all related elements of an event target (must be an element)
const getRelatedElements = (event: Event & { target: ElementState }) => gr(event.target);

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

/// Process mouse move event on pseudo-link elements
const linkmove = (elem: ElementState) =>
  ignoredEvent(() => gr(elem)?.forEach(e => e.classList.add('hover')), 200, 'mouse-move');

/// Process mouse leave event on pseudo-link elements
const linkleave = (elem: ElementState) => gr(elem)?.forEach(e => e.classList.remove('hover'));

const semaLinkEnter = (a: any, bound: any) => () => {
  const href =
    bound.parentElement?.getAttribute('href') || bound.parentElement?.getAttribute('xlink:href');
  if (a.getAttribute('href') !== href) {
    a.setAttribute('href', href || '');
  }
};

interface ProcessOptions {
  layoutText?: boolean;
}

// background: transparent;

interface TextFlowCache {
  flow: HTMLDivElement[];
}

window.typstProcessSvg = function (docRoot: SVGElement, options?: ProcessOptions) {
  var elements = docRoot.getElementsByClassName('pseudo-link');

  for (var i = 0; i < elements.length; i++) {
    var elem = elements[i];
    elem.addEventListener('mousemove', e => linkmove(e.target as any as ElementState));
    elem.addEventListener('mouseleave', e => linkleave(e.target as any as ElementState));
  }

  const layoutText = options?.layoutText ?? true;

  if (layoutText) {
    setTimeout(() => {
      // add rule: .tsel monospace
      // todo: outline styles
      const style = document.createElement('style');
      style.innerHTML = `.tsel { font-family: monospace; text-align-last: left !important; -moz-text-size-adjust: none; -webkit-text-size-adjust: none; text-size-adjust: none; overflow: hidden; }
.tsel span { position: relative !important; width: fit-content !important;  }`;
      document.getElementsByTagName('head')[0].appendChild(style);

      window.layoutText(docRoot);
    }, 0);
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

const LB = '\n'.codePointAt(0)!;

window.layoutText = async function (svg: Element) {
  const allElements = Array.from(
    svg.querySelectorAll('.tsel, .typst-content-hint, .pseudo-link'),
  ) as HTMLDivElement[];
  const layoutBegin = performance.now();
  const ctx = (
    document.createElementNS('http://www.w3.org/1999/xhtml', 'canvas') as HTMLCanvasElement
  ).getContext('2d')!;
  // 128 * 16 = 2048
  ctx.font = `128px monospace`;
  const enCharWidth = ctx.measureText('A').width;
  const offset = svg.getBoundingClientRect();
  const coordLeft = offset.left + window.scrollX;
  const coordTop = offset.top + window.scrollY;
  const resolveCoord = (elem: SVGGElement, x: number, y: number) => {
    var matrix = elem.getScreenCTM();

    if (!matrix) {
      return { x: 0, y: 0 };
    }

    return {
      x: matrix.a * x + matrix.c * y + matrix.e - coordLeft,
      y: matrix.b * x + matrix.d * y + matrix.f - coordTop,
    };
  };
  // console.log('width of single char', enCharWidth);

  let semanticContainer: HTMLDivElement | undefined;
  // insert exact after svg
  // svg may have no next sibling
  const sp = svg.parentElement;
  if (!sp) {
    semanticContainer = undefined;
  } else {
    if (svg.nextElementSibling?.classList.contains('typst-semantic-layer')) {
      semanticContainer = svg.nextElementSibling as HTMLDivElement;
    } else {
      semanticContainer = document.createElement('div');
      const svgWrapper = document.createElement('div');
      svgWrapper.style.position = 'relative';
      sp.replaceChild(svgWrapper, svg);
      svgWrapper.appendChild(svg);
      svgWrapper.appendChild(semanticContainer);

      semanticContainer.classList.add('typst-semantic-layer');
      semanticContainer.style.position = 'absolute';
      semanticContainer.style.left = '0';
      semanticContainer.style.top = '0';
      semanticContainer.style.zIndex = '1';
      semanticContainer.style.float = 'left';
      const svgWidth = svg.getAttribute('width');
      semanticContainer.style.width = `${svgWidth}px`;
      const svgHeight = svg.getAttribute('height');
      semanticContainer.style.height = `${svgHeight}px`;
      // semanticContainer.style.pointerEvents = 'all';
    }
  }

  // const textElements: { e: HTMLElement; x: number; y: number }[] = [];
  interface ParaBox extends DOMBox {}
  let paraBox: ParaBox = { left: 0, right: 0, bottom: 0, top: 0 };
  let paraBoxes: [HTMLSpanElement | null, ParaBox][] = [];
  const createTextElem = (elem: SVGGElement, primitive = 'span') => {
    const textElem = document.createElement(primitive);

    const bbox = elem.getBBox();
    const leftTop = resolveCoord(elem, bbox.x, bbox.y);
    const rightBottom = resolveCoord(elem, bbox.x + bbox.width, bbox.y + bbox.height);
    // const realBBox = {
    //   x: Math.min(leftTop.x, rightBottom.x),
    //   y: Math.min(leftTop.y, rightBottom.y),
    //   width: Math.abs(leftTop.x - rightBottom.x),
    //   height: Math.abs(leftTop.y - rightBottom.y),
    // };
    // console.log('realBBox', realBBox);
    // convert to global fixed position
    const xx = Math.min(leftTop.x, rightBottom.x);
    const x = xx + window.scrollX;
    const yy = Math.min(leftTop.y, rightBottom.y);
    const y = yy + window.scrollY;
    const width = Math.abs(leftTop.x - rightBottom.x);
    const height = Math.abs(leftTop.y - rightBottom.y);

    const halfH = height / 2;
    const paraBoxNew: ParaBox = {
      left: x - halfH,
      top: y - halfH,
      right: x + width + halfH,
      bottom: y + height + halfH,
    };
    if (overLappingBox(paraBox, paraBoxNew)) {
      paraBox.left = Math.min(paraBox.left, paraBoxNew.left);
      paraBox.top = Math.min(paraBox.top, paraBoxNew.top);
      paraBox.right = Math.max(paraBox.right, paraBoxNew.right);
      paraBox.bottom = Math.max(paraBox.bottom, paraBoxNew.bottom);
    } else {
      paraBoxes.push([textElem, paraBox]);
      paraBox = paraBoxNew;
    }

    // intersected with previous para box

    // textElem.style.border = '1px solid gray';

    textElem.classList.add('tsel');
    // textElem.style.position = 'relative';
    // textElem.style.display = 'inline-block';
    textElem.style.position = 'absolute';

    textElem.style.left = `${x}px`;
    textElem.style.top = `${y}px`;

    textElem.style.width = `${width}px`;
    textElem.style.height = `${height}px`;

    // textElements.push({ e: textElem, x, y });
    return textElem;
  };

  const isTablet = false;
  // const isTablet = true;

  const layoutRange = (tselSt: number, tselEd: number) => {
    const divs = allElements.slice(tselSt, tselEd);
    for (let d of divs) {
      const elem = d.parentElement! as any as SVGForeignObjectElement;

      if (semanticContainer) {
        if (d.classList.contains('typst-content-hint')) {
          const textElem = createTextElem(d as any as SVGGElement);
          textElem.style.fontSize = '0.1px';
          textElem.style.width = '0.1px';
          textElem.style.height = '0.1px';
          const hint = Number.parseInt(d.getAttribute('data-hint') || '0', 16) || LB;
          // encode hint as html entity
          textElem.innerHTML = hint === LB ? '<br/>' : `&#x${hint.toString(16)};`;
          semanticContainer.append(textElem);
          continue;
        } else if (d.classList.contains('pseudo-link')) {
          // mouse move binding
          const textElem = createTextElem(d as any as SVGGElement, 'a');
          textElem.style.cursor = 'pointer';
          textElem.addEventListener('mousemove', () => linkmove(d as any as ElementState));
          textElem.addEventListener('mouseleave', () => linkleave(d as any as ElementState));
          textElem.onclick = () => {
            d.dispatchEvent(new MouseEvent('click', { bubbles: true }));
          };
          // on mouse enter, change href
          textElem.addEventListener('mouseenter', () => {
            const href =
              d.parentElement?.getAttribute('href') || d.parentElement?.getAttribute('xlink:href');
            if (textElem.getAttribute('href') !== href) {
              textElem.setAttribute('href', href || '');
            }
          });
          semanticContainer.append(textElem);
          continue;
        }
      }

      if (d.style.fontSize) {
        const charContainers: HTMLSpanElement[] = [];
        const textContent = d.innerText;
        const relativeEnCharWidth = (enCharWidth * Number.parseFloat(d.style.fontSize)) / 128;
        if (!isTablet) {
          const glyphs = findGlyphListForText(d);
          if (!glyphs) {
            // console.log('no glyphs found...');
            continue;
          }
          const glyphLens = getGlyphLenShape(glyphs);
          const glyphAdvances = getGlyphAdvanceShape(glyphs).map(t => t / 16);

          let failed = false;
          let j = 0,
            k = 0,
            l = 0;
          let prevSpan: HTMLSpanElement | undefined = undefined;
          let prevAdvance = 0;
          for (let c of textContent) {
            // console.log('c', c, j, k, glyphAdvances);
            if (j >= glyphAdvances.length) {
              failed = true;
              break;
            }
            let advance = glyphAdvances[j];
            if (glyphLens[j] > 1) {
              advance += k * relativeEnCharWidth;
            }
            k++;
            if (k >= glyphLens[j]) {
              j++;
              k = 0;
            }

            const span = document.createElement('span');
            span.textContent = c;
            span.classList.add('tsel-tok');
            if (prevSpan) {
              prevSpan.style.letterSpacing = `${advance - prevAdvance - relativeEnCharWidth}px`;
            }
            prevSpan = span;
            prevAdvance = advance;
            charContainers.push(span);
            l += 1;
          }

          if (failed) {
            continue;
          }
        } else {
          const span = document.createElement('span');
          span.textContent = textContent;
          // calculate scalex
          const bbox = elem.getBBox();
          const realWidth = relativeEnCharWidth * textContent.length;
          if (elem) {
            const scalex = bbox.width / realWidth;
            span.style.display = 'inline-block';
            span.style.transform = `scaleX(${scalex})`;
            span.style.transformOrigin = 'left';
          }

          charContainers.push(span);
        }

        d.innerHTML = '';
        if (semanticContainer) {
          // inherit font size with scaling
          // textElem.style.fontSize = d.style.fontSize;
          const baseSize = Number.parseFloat(d.style.fontSize || '0');
          const scaledFontSize = Math.abs(
            resolveCoord(elem, 0, baseSize).y - resolveCoord(elem, 0, 0).y,
          );

          if (!isTablet) {
            const ratio = scaledFontSize / baseSize;
            for (let c of charContainers) {
              // letter spacing
              c.style.letterSpacing = `${
                Number.parseFloat(c.style.letterSpacing || '0') * ratio
              }px`;
            }
          }

          const textElem = createTextElem(elem);
          textElem.style.fontSize = `${scaledFontSize}px`;
          textElem.append(...charContainers);
          semanticContainer.append(textElem);
        } else {
          d.append(...charContainers);
        }
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
    await new Promise(resolve => {
      setTimeout(() => {
        layoutRange(chunkBegin, chunkBegin + chunkSize);
        resolve(undefined);
      });
    });
  }

  if (semanticContainer && paraBox.right != 0) {
    paraBoxes.push([null, paraBox]);
  }
  // get all elements in semantic container
  // if (semanticContainer?.children.length) {
  //   const perfBegin = performance.now();
  //   // retrieve all offset left and top at once to avoid reflow
  //   const elems = Array.from(textElements).map(({ e, x, y }) => {
  //     const offsetLeft = e.offsetLeft;
  //     const offsetTop = e.offsetTop;
  //     return { e, x, y, offsetLeft, offsetTop };
  //   });
  //   elems.forEach(({ e, x, y, offsetLeft, offsetTop }) => {
  //     //
  //     e.style.left = `${x - offsetLeft}px`;
  //     e.style.top = `${y - offsetTop}px`;
  //   });
  //   console.log(`relative positioning elements used since ${performance.now() - perfBegin} ms`);
  // }
  if (semanticContainer) {
    const perfBegin = performance.now();
    const colorRotate = ['red', 'green', 'blue', 'purple', 'orange'];
    let cnt = 0;
    for (let [elem, box] of paraBoxes) {
      if (cnt < paraBoxes.length - 1) {
        let nextBox = paraBoxes[cnt + 1][1];
        let leftLess = box.left < nextBox.left;
        let topLess = box.top < nextBox.top;
        let rightLess = box.right < nextBox.right;
        let bottomLess = box.bottom < nextBox.bottom;

        // adjust horizontal box
        if (leftLess != rightLess) {
          box.left = Math.min(box.left, nextBox.left);
          box.right = Math.max(box.right, nextBox.right);
        }
        // adjust vertical box
        if (topLess != bottomLess) {
          box.top = Math.min(box.top, nextBox.top);
          box.bottom = Math.max(box.bottom, nextBox.bottom);
        }
      }

      // elem.style.left = `${box.left}px`;
      // elem.style.top = `${box.top}px`;
      // elem.style.width = `${box.right - box.left}px`;
      // elem.style.height = `${box.bottom - box.top}px`;

      const paraSpan = document.createElement('span');
      paraSpan.style.zIndex = '-1';
      paraSpan.style.position = 'absolute';
      paraSpan.style.left = `${box.left}px`;
      paraSpan.style.top = `${box.top}px`;
      paraSpan.style.width = `${box.right - box.left}px`;
      paraSpan.style.height = `${box.bottom - box.top}px`;
      paraSpan.dir = 'ltr';
      paraSpan.style.unicodeBidi = 'isolated';
      // paraSpan.style.border = '1px solid red';
      // paraSpan.style.border = `1px solid ${colorRotate[cnt % colorRotate.length]}`;
      semanticContainer.insertBefore(paraSpan, elem);
      cnt++;
    }
    console.log(`layout paragraph used since ${performance.now() - perfBegin} ms`);
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
  if (elem.classList.contains('typst-semantic-layer')) {
    elem = elem.firstElementChild!;
    return elem && window.handleTypstLocation(elem, page, x, y, options);
  }
  const behavior = options?.behavior || 'smooth';
  const assignHashLoc =
    window.assignSemaHash ||
    ((u: number, x: number, y: number) => {
      // todo: multiple documents
      location.hash = `loc-${u}x${x.toFixed(2)}x${y.toFixed(2)}`;
    });
  // todo: abstraction
  let docRoot = findAncestor(elem, 'typst-doc');
  if (!docRoot) {
    docRoot = findAncestor(elem, 'typst-svg-page');
    if (!docRoot) {
      console.warn('no typst-doc or typst-svg-page found', elem);
      return;
    }
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

function findLinkInSvg(r: SVGSVGElement, xy: [number, number], target: any) {
  // children
  const bbox = r.getBoundingClientRect();
  if (
    xy[0] < bbox.left - 1 ||
    xy[0] > bbox.right + 1 ||
    xy[1] < bbox.top - 1 ||
    xy[1] > bbox.bottom + 1
  ) {
    return;
  }

  // foreignObject
  if (r.classList.contains('pseudo-link')) {
    return r;
  }

  for (let i = 0; i < r.children.length; i++) {
    const a = findLinkInSvg(r.children[i] as any as SVGSVGElement, xy, target) as SVGAElement;
    if (a) {
      return a;
    }
  }

  return undefined;
}

(window as any).typstBindSemantics = function (
  root: HTMLElement,
  svg: SVGSVGElement,
  semantics: HTMLDivElement,
) {
  if ('typstBindCustomSemantics' in window) {
    (window as any).typstBindCustomSemantics(root, svg, semantics);
  }

  semantics.addEventListener('mousemove', (event: MouseEvent) => {
    ignoredEvent(
      () => {
        // a link
        if ((event.target as any)?.tagName === 'A') {
          const target = event.target as any as any;
          if (target.cachedTarget) {
            return;
          }

          // console.log('svg typstBindSemantics', event.clientX, event.clientY, svg);
          const a = findLinkInSvg(svg, [event.clientX, event.clientY], event.target as any);
          // console.log('svg typstBindSemantics', a);
          if (a) {
            a.dispatchEvent(new MouseEvent('mousemove', { bubbles: true }));
            const sle = semaLinkEnter(a, target);
            target.addEventListener('mouseenter', () => {
              a.dispatchEvent(new MouseEvent('mouseenter', { bubbles: true }));
              sle();
            });
            target.addEventListener('mousemove', () => {
              a.dispatchEvent(new MouseEvent('mousemove', { bubbles: true }));
              linkmove(a);
            });
            target.addEventListener('mouseleave', () => {
              a.dispatchEvent(new MouseEvent('mouseleave', { bubbles: true }));
              linkleave(a);
            });
          }
        }
      },
      100,
      'mouseenter',
    );
  });
};
