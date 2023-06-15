// debounce https://stackoverflow.com/questions/23181243/throttling-a-mousemove-event-to-fire-no-more-than-5-times-a-second
// ignore fast events, good for capturing double click
// @param (callback): function to be run when done
// @param (delay): integer in milliseconds
// @param (id): string value of a unique event id
// @doc (event.timeStamp): http://api.jquery.com/event.timeStamp/
// @bug (event.currentTime): https://bugzilla.mozilla.org/show_bug.cgi?id=238041
var ignoredEvent = (function () {
  var last = {},
    diff,
    time;

  return function (callback, delay, id) {
    time = new Date().getTime();
    id = id || 'ignored event';
    diff = last[id] ? time - last[id] : time;

    if (diff > delay) {
      last[id] = time;
      callback();
    }
  };
})();

var elements = document.getElementsByClassName('pseudo-link');

var overLapping = function (a, b) {
  var aRect = a.getBoundingClientRect();
  var bRect = b.getBoundingClientRect();
  return !(
    aRect.right < bRect.left ||
    aRect.left > bRect.right ||
    aRect.bottom < bRect.top ||
    aRect.top > bRect.bottom
  );
};
var searchIntersections = function (root) {
  let parent = undefined,
    current = root;
  while (current) {
    if (current.classList.contains('group')) {
      parent = current;
      break;
    }
    current = current.parentElement;
  }
  if (!current) {
    console.log('no group found');
    return;
  }
  const group = parent;
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
var getRelatedElements = function (event) {
  let relatedElements = event.target.relatedElements;
  if (relatedElements === undefined || relatedElements === null) {
    relatedElements = event.target.relatedElements = searchIntersections(event.target);
  }
  return relatedElements;
};
var linkmove = function (event) {
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
var linkleave = function (event) {
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

for (var i = 0; i < elements.length; i++) {
  var elem = elements[i];
  elem.addEventListener('mousemove', linkmove);
  elem.addEventListener('mouseleave', linkleave);
}

function findAncestor(el, cls) {
  while ((el = el.parentElement) && !el.classList.contains(cls));
  return el;
}

window.handleTypstLocation = function (elem, page, x, y) {
  const docRoot = findAncestor(elem, 'typst-doc');
  const children = docRoot.children;
  let nthPage = 0;
  for (let i = 0; i < children.length; i++) {
    if (children[i].tagName === 'g') {
      nthPage++;
    }
    if (nthPage == page) {
      const page = children[i];
      const dataWidth = page.getAttribute('data-page-width');
      const dataHeight = page.getAttribute('data-page-height');
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

      console.log('scrolling to', xOffset, yOffset, left, top);

      window.scrollTo(xOffset, yOffset);
      const ripple = document.createElement('div');
      ripple.className = 'typst-ripple';
      docRoot.appendChild(ripple);

      ripple.style.left = left.toString() + 'px';
      ripple.style.top = top.toString() + 'px';

      ripple.style.animation = 'typst-ripple-effect .4s linear';
      ripple.onanimationend = () => {
        docRoot.removeChild(ripple);
      };
      return;
    }
  }
};
