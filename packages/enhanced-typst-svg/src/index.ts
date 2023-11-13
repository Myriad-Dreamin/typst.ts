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

window.typstProcessSvg = function (docRoot: Element) {
  var elements = docRoot.getElementsByClassName('pseudo-link');

  for (var i = 0; i < elements.length; i++) {
    var elem = elements[i];
    elem.addEventListener('mousemove', linkmove);
    elem.addEventListener('mouseleave', linkleave);
  }

  if (false) {
    setTimeout(() => {
      window.layoutText(docRoot);
    }, 0);
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

  docRoot.addEventListener('copy', (event: ClipboardEvent) => {
    const selection = getSelectedNodes(
      (n: Element) => n.classList?.contains('tsel') || n.classList?.contains('typst-content-hint'),
    ) as Element[];

    // for (let n of selection) {
    //   if (n.classList.contains('typst-group')) {
    //     console.log(n, (n as Element).getBoundingClientRect());
    //   }
    // }

    const textContent = [];
    let prevContent = false;
    for (let n of selection) {
      // console.log(n, (n as Element).getBoundingClientRect(), n.textContent);
      if (n.classList.contains('tsel')) {
        textContent.push(n.textContent);
        const r = n.getBoundingClientRect();
        prevContent = true;
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
    event.clipboardData!.setData('text/plain', text);
    event.preventDefault();
  });

  // let startMousePos: any = undefined;
  // document.addEventListener('selectstart', (event: any) => {
  //   startMousePos = { x: event.clientX, y: event.clientY };
  //   console.log(event, event.initEvent, Object.keys(event), Object.getOwnPropertyNames(event));
  // });

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
    div.style.left = (b.left + window.scrollX).toString();
    div.style.top = (b.top + window.scrollY).toString();
    div.style.width = b.width.toString();
    div.style.height = b.height.toString();
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
      return ca?.classList.contains('text-guard') || ca?.classList.contains('typst-page');
    };

    if (
      isPageGuardSelected(pickElem(rng.startContainer) as SVGGElement | null) ||
      isPageGuardSelected(pickElem(rng.endContainer) as SVGGElement | null)
    ) {
      console.log('page guard selected');
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
      (n: Element) => n.classList?.contains('tsel'),
    ) as Element[];

    // console.log(
    //   'selectionchange',
    //   rng,
    //   selectedTextList,
    //   start,
    //   rng.startOffset,
    //   end,
    //   rng.endOffset,
    // );

    const selRng = new Range();
    for (let n of selectedTextList) {
      if (n.classList.contains('tsel')) {
        const st = n === start ? rng.startOffset : 0;
        const ed = n === end ? rng.endOffset : -1;

        const np = findAncestor(n, 'typst-text')!;
        if (!np) {
          // console.log('no typst-text found...');
          continue;
        }
        const glyphRefs = Array.from(np.children).filter(e => e.tagName === 'use');
        if (!glyphRefs.length) {
          // console.log('no glyphs found...');
          continue;
        }

        let stGlyph = glyphRefs[0];
        let edGlyph: Element | undefined = undefined;

        const createSelGlyphs = () => {
          selRng.setStartBefore(stGlyph);
          if (edGlyph === undefined) {
            selRng.setEndAfter(glyphRefs[glyphRefs.length - 1]);
          } else {
            selRng.setEndBefore(edGlyph);
          }
          createSelBox(selBox!, selRng);
        };

        if (st === 0 && ed === -1) {
          // console.log('select all', stGlyph, edGlyph);
          createSelGlyphs();
          continue;
        }

        const glyphLens = glyphRefs.map(e => {
          const href = e.getAttribute('href')!;
          const e2 = document.getElementById(href.slice(1));
          return 1 + Number.parseInt(e2?.getAttribute('data-liga-len') || '0');
        });

        const findPos = (l: number) => {
          let pos = 0;
          for (let i = 0; i < glyphLens.length; i++) {
            if (pos + glyphLens[i] > l) {
              return glyphRefs[i];
            }
            pos += glyphLens[i];
          }
        };

        if (st !== 0) {
          stGlyph = findPos(st) || stGlyph;
        }

        if (ed !== -1) {
          edGlyph = findPos(ed);
        }

        // console.log('select', np, st, ed, stGlyph, edGlyph, glyphLens);
        createSelGlyphs();
      }
    }
  });

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

  const createPseudoText = (cls: string) => {
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
  };

  docRoot.querySelectorAll('.typst-page').forEach((e: Element) => {
    e.prepend(createPseudoText('text-guard'));
  });

  // docRoot.querySelectorAll('.typst-group').forEach((e: SVGGElement) => {
  //   console.log('typst-group', e, e.getBBox());
  //   const bbox = e.getBBox();
  //   const hit = createPseudoText('text-hit');
  //   const hit2 = hit.cloneNode(true) as Element;
  //   hit.setAttribute('width', '1');
  //   hit.setAttribute('x', bbox.width.toString());
  //   hit.setAttribute('height', bbox.height.toString());
  //   hit2.setAttribute('width', bbox.width.toString());
  //   hit2.setAttribute('y', bbox.height.toString());
  //   hit2.setAttribute('height', '1');
  //   e.append(hit, hit2);
  // });

  if (window.location.hash) {
    console.log('hash', window.location.hash);

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
  //   const divs = svg.querySelectorAll('.tsel');
  //   const ctx = document.createElementNS('http://www.w3.org/1999/xhtml', 'canvas').getContext('2d');
  //   const layoutBegin = performance.now();
  //   for (let d of divs) {
  //     if (d.getAttribute('data-typst-layout-checked')) {
  //       continue;
  //     }
  //     if (d.style.fontSize) {
  //       const foreignObj = d.parentElement;
  //       const innerText = d.innerText;
  //       const targetWidth = Number.parseFloat(foreignObj.getAttribute('width'));
  //       const currentX = Number.parseFloat(foreignObj.getAttribute('x')) || 0;
  //       ctx.font = `${d.style.fontSize} sans-serif`;
  //       const selfWidth = ctx.measureText(innerText).width;
  //       const scale = targetWidth / selfWidth;
  //       d.style.transform = `scaleX(${scale})`;
  //       foreignObj.setAttribute('width', selfWidth);
  //       foreignObj.setAttribute('x', currentX - (selfWidth - targetWidth) * 0.5);
  //       d.setAttribute('data-typst-layout-checked', '1');
  //     }
  //   }
  //   console.log(`layoutText used time ${performance.now() - layoutBegin} ms`);
};

window.handleTypstLocation = function (elem: Element, page: number, x: number, y: number) {
  const docRoot = findAncestor(elem, 'typst-doc');
  const children = docRoot.children;
  let nthPage = 0;
  for (let i = 0; i < children.length; i++) {
    if (children[i].tagName === 'g') {
      nthPage++;
    }
    if (nthPage == page) {
      const page = children[i];
      const dataWidth = Number.parseFloat(page.getAttribute('data-page-width')!);
      const dataHeight = Number.parseFloat(page.getAttribute('data-page-height')!);
      const rect = page.getBoundingClientRect();
      const xOffsetInner = Math.max(0, x / dataWidth - 0.05) * rect.width;
      const yOffsetInner = Math.max(0, y / dataHeight - 0.05) * rect.height;
      const xOffsetInnerFix = (x / dataWidth) * rect.width - xOffsetInner;
      const yOffsetInnerFix = (y / dataHeight) * rect.height - yOffsetInner;

      const docRoot = document.body || document.firstElementChild;
      const basePos = docRoot.getBoundingClientRect();

      const xOffset = rect.left - basePos.left + xOffsetInner;
      const yOffset = rect.top - basePos.top + yOffsetInner;
      const left = xOffset + xOffsetInnerFix;
      const top = yOffset + yOffsetInnerFix;

      window.scrollTo(xOffset, yOffset);

      triggerRipple(docRoot, left, top, 'typst-jump-ripple', 'typst-jump-ripple-effect .4s linear');
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
  ripple.style.left = left.toString() + 'px';
  ripple.style.top = top.toString() + 'px';

  docRoot.appendChild(ripple);

  ripple.style.animation = animation;
  ripple.onanimationend = () => {
    docRoot.removeChild(ripple);
  };
}

var scriptTag = document.currentScript;
if (scriptTag) {
  // console.log('new svg util updated 26  ');
  const docRoot = findAncestor(scriptTag, 'typst-doc');
  if (docRoot) {
    window.typstProcessSvg(docRoot);
  }
}
