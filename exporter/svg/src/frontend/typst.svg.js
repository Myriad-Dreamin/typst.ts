const ignoredEvent=function(){var e={},n,t;return function(o,u,i){t=new Date().getTime(),i=i||"ignored event",n=e[i]?t-e[i]:t,n&gt;u&&(e[i]=t,o())}}(),overLappingSimp=function(e,n){var t=e.getBoundingClientRect(),o=n.getBoundingClientRect();return!(t.right&lt;o.left||t.left&gt;o.right||t.bottom&lt;o.top||t.top&gt;o.bottom)},overLapping=function(e,n){var t=e.getBoundingClientRect(),o=n.getBoundingClientRect();return overLappingSimp(e,n)&&(Math.abs(t.left-o.left)+Math.abs(t.right-o.right))/Math.max(t.width,o.width)&lt;.5&&(Math.abs(t.bottom-o.bottom)+Math.abs(t.top-o.top))/Math.max(t.height,o.height)&lt;.5};var searchIntersections=function(e){let n,t=e;for(;t;){if(t.classList.contains("typst-group")){n=t;break}t=t.parentElement}if(!t){console.log("no group found");return}const u=n.children,i=u.length,a=[];for(let s=0;s&lt;i;s++){const l=u[s];overLapping(l,e)&&a.push(l)}return a},getRelatedElements=function(e){let n=e.target.relatedElements;return n==null&&(n=e.target.relatedElements=searchIntersections(e.target)),n},linkmove=function(e){ignoredEvent(function(){const n=getRelatedElements(e);if(n!=null)for(var t=0;t&lt;n.length;t++){var o=n[t];o.classList.contains("hover")||o.classList.add("hover")}},200,"mouse-move")},linkleave=function(e){const n=getRelatedElements(e);if(n!=null)for(var t=0;t&lt;n.length;t++){var o=n[t];o.classList.contains("hover")&&o.classList.remove("hover")}};function findAncestor(e,n){for(;e&&!e.classList.contains(n);)e=e.parentElement;return e}function findGlyphListForText(e){const n=findAncestor(e,"typst-text");if(n)return Array.from(n.children).filter(t=&gt;t.tagName==="use")}function nextNode(e){if(e.hasChildNodes())return e.firstChild;for(;e&&!e.nextSibling;)e=e.parentNode;return e?e.nextSibling:null}function getRangeSelectedNodes(e,n){var t=e.startContainer,o=e.endContainer;if(t==o){if(n(t))return[t];if(n(t.parentElement))return[t.parentElement]}for(var u=[];t&&t!=o;)t=nextNode(t),n(t)&&u.push(t);for(t=e.startContainer;t&&t!=e.commonAncestorContainer;)n(t)&&u.unshift(t),t=t.parentNode;return u}function getSelectedNodes(e){if(window.getSelection){var n=window.getSelection();if(!n.isCollapsed)return getRangeSelectedNodes(n.getRangeAt(0),e)}return[]}function getGlyphLenShape(e){return e.map(n=&gt;{const t=n.getAttribute("href"),o=document.getElementById(t.slice(1));return 1+Number.parseInt(o?.getAttribute("data-liga-len")||"0")})}function getGlyphAdvanceShape(e){return e.map(n=&gt;Number.parseInt(n.getAttribute("x")||"0"))}function adjsutTextSelection(e){e.addEventListener("copy",i=&gt;{const a=getSelectedNodes(d=&gt;d.classList?.contains("tsel")||d.classList?.contains("tsel-tok")||d.classList?.contains("typst-content-hint")),s=[];let l=!1;for(let d of a)if(d.classList.contains("tsel"))d.hasAttribute("data-typst-layout-checked")||s.push(d.textContent),l=!0;else if(d.classList.contains("tsel-tok"))s.push(d.textContent);else if(l){const p=String.fromCodePoint(Number.parseInt(d.getAttribute("data-hint")||"0",16))||`
`;s.push(p),l=!0}const r=s.join("").replace(/\u00a0/g," ");navigator?.clipboard?navigator.clipboard.writeText(r):i.clipboardData.setData("text/plain",r),i.preventDefault()});const n=i=&gt;i.nodeType===Node.TEXT_NODE?i.parentElement:i,t=i=&gt;{const a=n(i);return a?.classList?.contains("tsel")?a:void 0},o=(i,a)=&gt;{const s=document.createElement("div"),l=a.getBoundingClientRect();s.style.position="absolute",s.style.float="left",s.style.left=(l.left+window.scrollX).toString(),s.style.top=(l.top+window.scrollY).toString(),s.style.width=l.width.toString(),s.style.height=l.height.toString(),s.style.backgroundColor="#7db9dea0",i.appendChild(s)},u=i=&gt;{i&&(i.innerHTML="")};document.addEventListener("selectionchange",i=&gt;{const a=window.getSelection();let s=document.getElementById("tsel-sel-box");if(!a?.rangeCount){u(s);return}const l=a?.getRangeAt(0);if(!l)return;const r=c=&gt;c?.classList.contains("text-guard")||c?.classList.contains("typst-page")||c?.classList.contains("typst-search-hint"),d=r(n(l.startContainer)),p=r(n(l.endContainer));if(d||p){console.log("page guard selected"),d&&p&&u(s);return}u(s),s||(s=document.createElement("div"),s.id="tsel-sel-box",s.style.zIndex="100",s.style.position="absolute",s.style.pointerEvents="none",s.style.left="0",s.style.top="0",s.style.float="left",document.body.appendChild(s));const y=t(l.startContainer),E=t(l.endContainer),C=getSelectedNodes(c=&gt;c.classList?.contains("tsel")||c.classList?.contains("typst-search-hint")||c.classList?.contains("tsel-tok")),b=new Range,v=(c,f)=&gt;{b.setStartBefore(c),b.setEndAfter(f),o(s,b)},g=new Map;for(let c of C)if(c.classList.contains("tsel-tok")){const f=c.parentElement,m=Array.from(f.children).indexOf(c);if(!g.has(f))g.set(f,[m,m]);else{const[h,w]=g.get(f);g.set(f,[Math.min(h,m),Math.max(w,m)])}}else if(c.classList.contains("tsel")&&!c.hasAttribute("data-typst-layout-checked")){const f=c===y?l.startOffset:0,m=c===E?l.endOffset-1:-1;g.set(c,[f,m])}for(let[c,[f,m]]of g){const h=findGlyphListForText(c);if(!h?.length)continue;if(f===0&&m===-1){v(h[0],h[h.length-1]);continue}const w=getGlyphLenShape(h),L=A=&gt;{let N=0;for(let S=0;S&lt;w.length;S++){if(N+w[S]&gt;A)return h[S];N+=w[S]}};let x=h[0];f!==0&&(x=L(f)||x);let T=h[h.length-1];m!==-1&&(T=L(m)||T),v(x,T)}})}function createPseudoText(e){const n=document.createElementNS("http://www.w3.org/2000/svg","foreignObject");n.setAttribute("width","1"),n.setAttribute("height","1"),n.setAttribute("x","0"),n.setAttribute("y","0");const t=document.createElement("span");return t.textContent="&nbsp;",t.style.width=t.style.height="100%",t.style.textAlign="justify",t.style.opacity="0",t.classList.add(e),n.append(t),n}window.typstProcessSvg=function(e,n){for(var t=e.getElementsByClassName("pseudo-link"),o=0;o&lt;t.length;o++){var u=t[o];u.addEventListener("mousemove",linkmove),u.addEventListener("mouseleave",linkleave)}const i=n?.layoutText??!0;if(i&&(setTimeout(()=&gt;{const a=document.createElement("style");a.innerHTML=`.tsel { font-family: monospace; text-align-last: left !important; -moz-text-size-adjust: none; -webkit-text-size-adjust: none; text-size-adjust: none; }
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
} `,document.getElementsByTagName("head")[0].appendChild(a);const s=window.devicePixelRatio||1;e.style.setProperty("--typst-font-scale",s.toString()),window.addEventListener("resize",()=&gt;{const l=window.devicePixelRatio||1;e.style.setProperty("--typst-font-scale",l.toString())}),window.layoutText(e)},0),adjsutTextSelection(e)),e.addEventListener("click",a=&gt;{let s=a.target;for(;s;){const l=s.getAttribute("data-span");if(l){console.log("source-span of this svg element",l);const r=document.body||document.firstElementChild,d=r.getBoundingClientRect(),p=window.innerWidth||0,y=a.clientX-d.left+.015*p,E=a.clientY-d.top+.015*p;triggerRipple(r,y,E,"typst-debug-react-ripple","typst-debug-react-ripple-effect .4s linear");return}s=s.parentElement}}),i&&e.querySelectorAll(".typst-page").forEach(a=&gt;{a.prepend(createPseudoText("text-guard"))}),window.location.hash){const s=window.location.hash.split("-");if(s.length===2&&s[0]==="#loc"){const l=s[1].split("x");if(l.length===3){const r=Number.parseInt(l[0]),d=Number.parseFloat(l[1]),p=Number.parseFloat(l[2]);window.handleTypstLocation(e,r,d,p)}}}},window.layoutText=function(e){const n=Array.from(e.querySelectorAll(".tsel")),t=performance.now(),o=document.createElementNS("http://www.w3.org/1999/xhtml","canvas").getContext("2d");o.font="128px sans-serif";const u=o.measureText("A").width,i=[],a=l=&gt;{for(let r of l)if(!r.getAttribute("data-typst-layout-checked")&&r.style.fontSize){const d=r.parentElement,p=r.innerText,y=d.cloneNode(!0),E=y.firstElementChild;E&&(E.className="typst-search-hint"),d.parentElement.insertBefore(y,d),i.push([r,p]);const C=p.length,b=findGlyphListForText(r);if(!b)continue;const v=getGlyphLenShape(b),g=getGlyphAdvanceShape(b).map(w=&gt;w/16);let c=!1;const f=[];let m=0,h=0;for(let w of p){if(m&gt;=g.length){c=!0;break}let L=g[m];v[m]&gt;1&&(L+=h*u),h++,h&gt;=v[m]&&(m++,h=0);const x=document.createElement("span");x.textContent=w,x.classList.add("tsel-tok"),x.style.left=L.toString(),f.push(x)}if(c)continue;r.innerHTML="",r.append(...f),r.setAttribute("data-typst-layout-checked","1")}console.log(`layoutText ${n.length} elements used since ${performance.now()-t} ms`)},s=100;for(let l=0;l&lt;n.length;l+=s){const r=l;setTimeout(()=&gt;{a(n.slice(r,r+s))})}},window.handleTypstLocation=function(e,n,t,o){const i=findAncestor(e,"typst-doc").children;let a=0;for(let s=0;s&lt;i.length;s++)if(i[s].tagName==="g"&&a++,a==n){const l=i[s],r=Number.parseFloat(l.getAttribute("data-page-width")),d=Number.parseFloat(l.getAttribute("data-page-height")),p=l.getBoundingClientRect(),y=Math.max(0,t/r-.05)*p.width,E=Math.max(0,o/d-.05)*p.height,C=t/r*p.width-y,b=o/d*p.height-E,v=document.body||document.firstElementChild,g=v.getBoundingClientRect(),c=p.left-g.left+y,f=p.top-g.top+E,m=c+C,h=f+b;window.scrollTo(c,f),triggerRipple(v,m,h,"typst-jump-ripple","typst-jump-ripple-effect .4s linear");return}};function triggerRipple(e,n,t,o,u){const i=document.createElement("div");i.className=o,i.style.left=n.toString()+"px",i.style.top=t.toString()+"px",e.appendChild(i),i.style.animation=u,i.onanimationend=()=&gt;{e.removeChild(i)}}var scriptTag=document.currentScript;if(scriptTag){const e=findAncestor(scriptTag,"typst-doc");e&&window.typstProcessSvg(e)}
