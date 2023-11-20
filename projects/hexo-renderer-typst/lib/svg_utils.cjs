const ignoredEvent=function(){var e={},o,s;return function(l,E,c){s=new Date().getTime(),c=c||"ignored event",o=e[c]?s-e[c]:s,o>E&&(e[c]=s,l())}}(),overLappingSimp=function(e,o){var s=e.getBoundingClientRect(),l=o.getBoundingClientRect();return!(s.right<l.left||s.left>l.right||s.bottom<l.top||s.top>l.bottom)},overLapping=function(e,o){var s=e.getBoundingClientRect(),l=o.getBoundingClientRect();return overLappingSimp(e,o)&&(Math.abs(s.left-l.left)+Math.abs(s.right-l.right))/Math.max(s.width,l.width)<.5&&(Math.abs(s.bottom-l.bottom)+Math.abs(s.top-l.top))/Math.max(s.height,l.height)<.5};var searchIntersections=function(e){let o,s=e;for(;s;){if(s.classList.contains("typst-group")){o=s;break}s=s.parentElement}if(!s){console.log("no group found");return}const E=o.children,c=E.length,v=[];for(let f=0;f<c;f++){const h=E[f];overLapping(h,e)&&v.push(h)}return v},getRelatedElements=function(e){let o=e.target.relatedElements;return o==null&&(o=e.target.relatedElements=searchIntersections(e.target)),o},linkmove=function(e){ignoredEvent(function(){const o=getRelatedElements(e);if(o!=null)for(var s=0;s<o.length;s++){var l=o[s];l.classList.contains("hover")||l.classList.add("hover")}},200,"mouse-move")},linkleave=function(e){const o=getRelatedElements(e);if(o!=null)for(var s=0;s<o.length;s++){var l=o[s];l.classList.contains("hover")&&l.classList.remove("hover")}};function findAncestor(e,o){for(;e&&!e.classList.contains(o);)e=e.parentElement;return e}function findGlyphListForText(e){const o=findAncestor(e,"typst-text");if(o)return Array.from(o.children).filter(s=>s.tagName==="use")}function getGlyphLenShape(e){return e.map(o=>{const s=o.getAttribute("href"),l=document.getElementById(s.slice(1));return 1+Number.parseInt(l?.getAttribute("data-liga-len")||"0")})}function getGlyphAdvanceShape(e){return e.map(o=>Number.parseInt(o.getAttribute("x")||"0"))}window.typstProcessSvg=function(e){for(var o=e.getElementsByClassName("pseudo-link"),s=0;s<o.length;s++){var l=o[s];l.addEventListener("mousemove",linkmove),l.addEventListener("mouseleave",linkleave)}setTimeout(()=>{const n=document.createElement("style");n.innerHTML=`.tsel { font-family: monospace; text-align-last: left !important; -moz-text-size-adjust: none; -webkit-text-size-adjust: none; text-size-adjust: none; }
.tsel span { float: left !important; position: absolute !important; width: fit-content !important; top: 0 !important; }
.typst-search-hint { font-size: 2048px; color: transparent; width: 100%; height: 100%; }
.typst-search-hint { color: transpaent; user-select: none; }
.typst-search-hint::-moz-selection { color: transpaent; background: #00000001; }
.typst-search-hint::selection { color: transpaent; background: #00000001; } `,document.getElementsByTagName("head")[0].appendChild(n);const i=window.devicePixelRatio||1;e.style.setProperty("--typst-font-scale",i.toString()),window.addEventListener("resize",()=>{const t=window.devicePixelRatio||1;e.style.setProperty("--typst-font-scale",t.toString())}),window.layoutText(e)},0);function E(n){if(n.hasChildNodes())return n.firstChild;for(;n&&!n.nextSibling;)n=n.parentNode;return n?n.nextSibling:null}function c(n,i){var t=n.startContainer,a=n.endContainer;if(t==a){if(i(t))return[t];if(i(t.parentElement))return[t.parentElement]}for(var d=[];t&&t!=a;)t=E(t),i(t)&&d.push(t);for(t=n.startContainer;t&&t!=n.commonAncestorContainer;)i(t)&&d.unshift(t),t=t.parentNode;return d}function v(n){if(window.getSelection){var i=window.getSelection();if(!i.isCollapsed)return c(i.getRangeAt(0),n)}return[]}e.addEventListener("copy",n=>{const i=v(r=>r.classList?.contains("tsel")||r.classList?.contains("tsel-tok")||r.classList?.contains("typst-content-hint")),t=[];let a=!1;for(let r of i)if(r.classList.contains("tsel"))r.hasAttribute("data-typst-layout-checked")||t.push(r.textContent),a=!0;else if(r.classList.contains("tsel-tok"))t.push(r.textContent);else if(a){const y=String.fromCodePoint(Number.parseInt(r.getAttribute("data-hint")||"0",16))||`
`;t.push(y),a=!0}const d=t.join("").replace(/\u00a0/g," ");navigator?.clipboard?navigator.clipboard.writeText(d):n.clipboardData.setData("text/plain",d),n.preventDefault()});const f=n=>n.nodeType===Node.TEXT_NODE?n.parentElement:n,h=n=>{const i=f(n);return i?.classList?.contains("tsel")?i:void 0},p=(n,i)=>{const t=document.createElement("div"),a=i.getBoundingClientRect();t.style.position="absolute",t.style.float="left",t.style.left=(a.left+window.scrollX).toString(),t.style.top=(a.top+window.scrollY).toString(),t.style.width=a.width.toString(),t.style.height=a.height.toString(),t.style.backgroundColor="#7db9dea0",n.appendChild(t)},x=n=>{n&&(n.innerHTML="")};document.addEventListener("selectionchange",n=>{const i=window.getSelection();let t=document.getElementById("tsel-sel-box");if(!i?.rangeCount){x(t);return}const a=i?.getRangeAt(0);if(!a)return;const d=u=>u?.classList.contains("text-guard")||u?.classList.contains("typst-page")||u?.classList.contains("typst-search-hint"),r=d(f(a.startContainer)),y=d(f(a.endContainer));if(r||y){console.log("page guard selected"),r&&y&&x(t);return}x(t),t||(t=document.createElement("div"),t.id="tsel-sel-box",t.style.zIndex="100",t.style.position="absolute",t.style.pointerEvents="none",t.style.left="0",t.style.top="0",t.style.float="left",document.body.appendChild(t));const C=h(a.startContainer),S=h(a.endContainer),N=v(u=>u.classList?.contains("tsel")||u.classList?.contains("typst-search-hint")||u.classList?.contains("tsel-tok")),T=new Range,A=(u,m)=>{T.setStartBefore(u),T.setEndAfter(m),p(t,T)},b=new Map;for(let u of N)if(u.classList.contains("tsel-tok")){const m=u.parentElement,L=Array.from(m.children).indexOf(u);if(!b.has(m))b.set(m,[L,L]);else{const[w,R]=b.get(m);b.set(m,[Math.min(w,L),Math.max(R,L)])}}else if(u.classList.contains("tsel")&&!u.hasAttribute("data-typst-layout-checked")){const m=u===C?a.startOffset:0,L=u===S?a.endOffset-1:-1;b.set(u,[m,L])}for(let[u,[m,L]]of b){const w=findGlyphListForText(u);if(!w?.length)continue;if(m===0&&L===-1){A(w[0],w[w.length-1]);continue}const R=getGlyphLenShape(w),B=I=>{let G=0;for(let k=0;k<R.length;k++){if(G+R[k]>I)return w[k];G+=R[k]}};let M=w[0];m!==0&&(M=B(m)||M);let P=w[w.length-1];L!==-1&&(P=B(L)||P),A(M,P)}}),e.addEventListener("click",n=>{let i=n.target;for(;i;){const t=i.getAttribute("data-span");if(t){console.log("source-span of this svg element",t);const a=document.body||document.firstElementChild,d=a.getBoundingClientRect(),r=window.innerWidth||0,y=n.clientX-d.left+.015*r,C=n.clientY-d.top+.015*r;triggerRipple(a,y,C,"typst-debug-react-ripple","typst-debug-react-ripple-effect .4s linear");return}i=i.parentElement}});const g=n=>{const i=document.createElementNS("http://www.w3.org/2000/svg","foreignObject");i.setAttribute("width","1"),i.setAttribute("height","1"),i.setAttribute("x","0"),i.setAttribute("y","0");const t=document.createElement("span");return t.textContent="&nbsp;",t.style.width=t.style.height="100%",t.style.textAlign="justify",t.style.opacity="0",t.classList.add(n),i.append(t),i};if(e.querySelectorAll(".typst-page").forEach(n=>{n.prepend(g("text-guard"))}),window.location.hash){const i=window.location.hash.split("-");if(i.length===2&&i[0]==="#loc"){const t=i[1].split("x");if(t.length===3){const a=Number.parseInt(t[0]),d=Number.parseFloat(t[1]),r=Number.parseFloat(t[2]);window.handleTypstLocation(e,a,d,r)}}}},window.layoutText=function(e){const o=Array.from(e.querySelectorAll(".tsel")),s=performance.now(),l=document.createElementNS("http://www.w3.org/1999/xhtml","canvas").getContext("2d");l.font="128px sans-serif";const E=l.measureText("A").width,c=[],v=h=>{for(let p of h)if(!p.getAttribute("data-typst-layout-checked")&&p.style.fontSize){const x=p.parentElement,g=p.innerText,n=x.cloneNode(!0),i=n.firstElementChild;i&&(i.className="typst-search-hint"),x.parentElement.insertBefore(n,x),c.push([p,g]);const t=g.length,a=findGlyphListForText(p);if(!a)continue;const d=getGlyphLenShape(a),r=getGlyphAdvanceShape(a).map(T=>T/16);let y=!1;const C=[];let S=0,N=0;for(let T of g){if(S>=r.length){y=!0;break}let A=r[S];d[S]>1&&(A+=N*E),N++,N>=d[S]&&(S++,N=0);const b=document.createElement("span");b.textContent=T,b.classList.add("tsel-tok"),b.style.left=A.toString(),C.push(b)}if(y)continue;p.innerHTML="",p.append(...C),p.setAttribute("data-typst-layout-checked","1")}console.log(`layoutText ${o.length} elements used since ${performance.now()-s} ms`)},f=100;for(let h=0;h<o.length;h+=f){const p=h;setTimeout(()=>{v(o.slice(p,p+f))})}},window.handleTypstLocation=function(e,o,s,l){const c=findAncestor(e,"typst-doc").children;let v=0;for(let f=0;f<c.length;f++)if(c[f].tagName==="g"&&v++,v==o){const h=c[f],p=Number.parseFloat(h.getAttribute("data-page-width")),x=Number.parseFloat(h.getAttribute("data-page-height")),g=h.getBoundingClientRect(),n=Math.max(0,s/p-.05)*g.width,i=Math.max(0,l/x-.05)*g.height,t=s/p*g.width-n,a=l/x*g.height-i,d=document.body||document.firstElementChild,r=d.getBoundingClientRect(),y=g.left-r.left+n,C=g.top-r.top+i,S=y+t,N=C+a;window.scrollTo(y,C),triggerRipple(d,S,N,"typst-jump-ripple","typst-jump-ripple-effect .4s linear");return}};function triggerRipple(e,o,s,l,E){const c=document.createElement("div");c.className=l,c.style.left=o.toString()+"px",c.style.top=s.toString()+"px",e.appendChild(c),c.style.animation=E,c.onanimationend=()=>{e.removeChild(c)}}var scriptTag=document.currentScript;if(scriptTag){const e=findAncestor(scriptTag,"typst-doc");e&&window.typstProcessSvg(e)}
