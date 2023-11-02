import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';

// Use CDN
let compiler = fetch(
  'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
);
let renderer = fetch(
  'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
);

// Use local server
// let compiler = fetch(
//   'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
// );
// let renderer = fetch(
//   'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
// );

// Bundle
// @ts-ignore
// import compiler from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
// @ts-ignore
// import renderer from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';

$typst.setCompilerInitOptions({
  getModule: () => compiler,
});
$typst.setRendererInitOptions({
  getModule: () => renderer,
});

const INEW_LINE = '\n  ';

interface ElementDefinition {
  id: string;
  interactive?: boolean;
  draw: string;
  sig: string;
  interactiveSig: string;
  props?: Record<string, any>;
}

interface EditorState {
  definitions?: ElementDefinition[];
  width?: string;
  height?: string;
  main?: any;
}

class PreviewState {
  definitions: Map<string, ElementDefinition> = new Map();
  previewingDef: string = '';
  isMain = false;
  width: number = 500;
  height: number = 500;
  main: Record<string, any> = {};
  element: HTMLElement | null = null;
  selectedElement: SVGElement | null = null;
  svg: SVGSVGElement | null = null;
  offset: any;
  transform: any;
  updateMainContent: (content: string) => void = undefined!;
  constructor() {}

  bindElement(element: HTMLElement) {
    this.element = element;
    this.element.addEventListener('click', e => {});
    this.element.addEventListener('mousedown', e => this.startDrag(e));
    this.element.addEventListener('mousemove', e => this.drag(e));
    this.element.addEventListener('mouseup', e => this.endDrag(e));
    this.element.addEventListener('mouseleave', e => this.endDrag(e));
    element.addEventListener('contextmenu', e => {
      e.preventDefault();
      this.checkContextMenuAction(e);
    });
    this.flushPreview();

    this.updateMainContent = (window as any).updateMainContent;
  }

  // exportSvg();
  async exportSvg() {
    const d = await this.exportAs('svg');
    var b = new Blob([d], { type: 'image/svg' });
    this.exportBlobTo(b);
  }

  async exportPdf() {
    const d = await this.exportAs('pdf');
    var b = new Blob([d], { type: 'application/pdf' });
    this.exportBlobTo(b);
  }

  async exportCetz() {
    const d = await this.exportAs('cetz');
    var b = new Blob([d], { type: 'text/plain' });
    this.exportBlobTo(b);
  }

  private exportBlobTo(blob: Blob) {
    // Create element with <a> tag
    const link = document.createElement('a');

    // Add file content in the object URL
    link.href = URL.createObjectURL(blob);

    // Add file name
    link.target = '_blank';

    // Add click event to <a> tag to save file.
    link.click();
    URL.revokeObjectURL(link.href);
  }

  previewPromise: Promise<void> | null = null;
  flushPreview() {
    if (this.previewPromise) {
      this.previewPromise = this.previewPromise
        .then(() => this.workPreview())
        .catch(e => console.log(e));
    } else {
      this.previewPromise = this.workPreview();
    }
  }

  async workPreview() {
    if (!this.element) {
      return;
    }
    const sigs: string[] = [];
    this.definitions.forEach(def => {
      sigs.push(def.sig);
    });

    let content: string | undefined = undefined;

    let isMain = () => false;
    if (this.previewingDef === '' || this.previewingDef == 'main') {
      // console.log('previewing main', $typst, this);
      content = await this.drawInteractive();
      isMain = () => true;
    } else {
      const def = this.definitions.get(this.previewingDef);
      if (def) {
        // console.log('previewing definition', def, $typst, this);
        content = await this.drawDefinition(def);
      }
    }

    if (content !== undefined) {
      // console.log({ content });
      this.element.innerHTML = content;

      const svgElem = this.element.firstElementChild;
      this.svg = svgElem as any;
      if (!svgElem) {
        return;
      }
      const width = Number.parseFloat(svgElem.getAttribute('width')!);
      const height = Number.parseFloat(svgElem.getAttribute('height')!);
      const cw = document.body.clientWidth / 2 - 40;
      svgElem.setAttribute('width', cw.toString());
      svgElem.setAttribute('height', ((height * cw) / width).toString());
      this.isMain = isMain();
    }
  }

  find(target: Element) {
    while (target) {
      if (target.classList.contains('typst-cetz-elem')) {
        return target;
      }
      target = target.parentElement!;
    }
    return undefined;
  }

  getMousePosition(evt: MouseEvent) {
    var CTM = this.svg!.getScreenCTM()!;
    return {
      x: (evt.clientX - CTM.e) / CTM.a,
      y: (evt.clientY - CTM.f) / CTM.d,
    };
  }

  startDrag(evt: MouseEvent) {
    if (!this.isMain) {
      return;
    }

    let target = this.find(evt.target as Element);
    if (target) {
      const elem = (this.selectedElement = target as any);

      this.offset = this.getMousePosition(evt);
      // Get all the transforms currently on this element
      let transforms = elem.transform.baseVal;
      // Ensure the first transform is a translate transform
      if (
        transforms.length === 0 ||
        transforms.getItem(0).type !== SVGTransform.SVG_TRANSFORM_TRANSLATE
      ) {
        // Create an transform that translates by (0, 0)
        var translate = this.svg!.createSVGTransform();
        translate.setTranslate(0, 0);
        // Add the translation to the front of the transforms list
        elem.transform.baseVal.insertItemBefore(translate, 0);
      }
      // Get initial translation amount
      this.transform = transforms.getItem(0);
      this.offset.x -= this.transform.matrix.e;
      this.offset.y -= this.transform.matrix.f;

      const typstId = target.id.replace('cetz-app-', '');
      if (!this.main[typstId].initPos) {
        this.main[typstId].initPos = [...this.main[typstId].pos];
      }
    } else {
      this.selectedElement = null;
    }
  }

  syncMainContent() {
    const data: string[] = [];
    for (const [_, ins] of Object.entries(this.main).sort((x, y) => {
      return x[1].idx - y[1].idx;
    })) {
      let args = '';
      const argEntries = Object.entries(ins.args ?? {});
      if (argEntries.length) {
        args = '  args:';
        for (const [k, v] of argEntries) {
          args += `\n    ${k}: ${v}`;
        }
      }
      let [x, y] = ins.pos;
      if (ins.deltaPos) {
        x = ins.initPos[0] + ins.deltaPos[0];
        y = ins.initPos[1] - ins.deltaPos[1];
      }
      data.push(`- name: ${ins.name}\n  type: ${ins.type}${args}\n  pos: [${x}, ${y}]`);
    }
    this.updateMainContent(data.join('\n'));
  }

  drag(evt: MouseEvent) {
    const selectedElement = this.selectedElement;
    if (selectedElement) {
      evt.preventDefault();
      var coord = this.getMousePosition(evt);
      const x = coord.x - this.offset.x;
      const y = coord.y - this.offset.y;
      this.transform.setTranslate(x, y);
      const typstId = selectedElement.id.replace('cetz-app-', '');
      this.main[typstId].deltaPos = [x, y];
      this.syncMainContent();
    }
  }

  endDrag(e: MouseEvent) {
    if (this.selectedElement) {
      const typstId = this.selectedElement.id.replace('cetz-app-', '');
      const ins = this.main[typstId];
      if (ins.deltaPos) {
        ins.pos[0] = ins.initPos[0] + ins.deltaPos[0];
        ins.pos[1] = ins.initPos[1] - ins.deltaPos[1];
        ins.deltaPos = undefined;
        ins.initPos = undefined;
        console.log(JSON.stringify(ins));
        setTimeout(() => {
          this.flushPreview();
        }, 16);
      }
      this.selectedElement = null;
    }
  }

  checkContextMenuAction(e: MouseEvent) {
    let target = this.find(e.target as Element);
    if (!target) {
      return;
    }

    e.preventDefault();

    console.log('checkClickAction', target.id);

    const typstId = target.id.replace('cetz-app-', '');
    const ins = this.main[typstId];
    const def = this.definitions.get(ins.type)!;
    if (!def || def.interactive === false) {
      return;
    }

    const x = e.clientX;
    const y = e.clientY;

    const menu = document.getElementById('svg-edit-menu')!;
    menu.innerHTML = '';

    // Set the position for menu
    menu.style.top = `${y}px`;
    menu.style.left = `${x}px`;

    menu.style.width = `auto`;

    console.log(def.props, ins.props);
    for (const [k, v] of Object.entries(def.props ?? {})) {
      const div = document.createElement('div');
      div.classList.add('menu-item');
      const labelSpan = document.createElement('span');
      labelSpan.innerText = `${k}: `;
      div.appendChild(labelSpan);

      const input = document.createElement('input');
      input.type = 'text';
      input.value = ins.args?.[k] ?? v ?? '';
      input.placeholder = k;
      input.addEventListener('input', e => {
        const inputValue = (e.target! as HTMLInputElement).value;
        console.log('change', inputValue);
        ins.args = ins.args ?? {};
        ins.args[k] = inputValue;
        this.flushPreview();
      });
      div.appendChild(input);

      const clearBoth = document.createElement('div');
      clearBoth.style.clear = 'both';
      div.appendChild(clearBoth);
      menu.appendChild(div);
    }

    // Show the menu
    menu.classList.toggle('hidden');
  }

  async drawInteractive() {
    //     const instances: string[] = [];
    //     for (const k of Object.keys(this.main)) {
    //       const ins = this.main[k];
    //       // const pos = `(${ins.pos[0]}, ${ins.pos[1]})`;
    //       const def = await this.drawDefinition(this.definitions.get(ins.type)!, ins.args?.trim());
    //       instances.push(
    //         `<g id="cetz-app-${k}" class="typst-cetz-elem" transform="translate(${ins.pos[0]}, ${ins.pos[1]})">${def}</g>`,
    //       );
    //     }

    //     return `<svg viewBox="0 0 ${this.width} ${this.height}" width="${this.width}" height="${
    //       this.height
    //     }" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" xmlns:h5="http://www.w3.org/1999/xhtml">
    //   ${instances.join(INEW_LINE)}
    // </svg>`;

    return (await this.exportAs('svg', true)) as string;
  }

  getDefinitionNames() {
    return Array.from(this.definitions.keys());
  }

  async exportAs(kind?: string, interactive?: boolean) {
    const instances: string[] = [];
    let prevPos = [0, 0];
    let t = 1;
    const tagMapping: string[] = [];
    for (const [k, ins] of Object.entries(this.main).sort((x, y) => {
      return x[1].idx - y[1].idx;
    })) {
      let args = `node-label: "${k}"`;
      for (const [k, v] of Object.entries(ins.args ?? {})) {
        args += `, ${k}: ${v}`;
      }
      let tag = '';
      if (interactive) {
        tag = `, tag: rgb("#${t.toString(16).padStart(6, '0')}")`;
        t++;
        tagMapping.push(k);
      }
      instances.push(
        `translate((${ins.pos[0] - prevPos[0]}, ${ins.pos[1] - prevPos[1]}))\n  ${
          ins.type
        }(${args}${tag}) // ${k}`,
      );
      prevPos = ins.pos;
    }

    const mainContent = `
#import "@preview/cetz:0.1.2"
#set page(margin: 1pt, width: ${this.width + 2}pt, height: ${this.height + 2}pt)
#let debug-label(_) = ()
#cetz.canvas({
import cetz.draw: *
  ${this.sigs(interactive).join(INEW_LINE)}
  ${instances.join(INEW_LINE)}
}, length: 1pt)
`;
    // console.log({ mainContent });

    switch (kind) {
      case 'svg':
        return this.postProcess(await $typst.svg({ mainContent }), tagMapping);
      case 'pdf':
        return await $typst.pdf({ mainContent });
      case 'cetz':
      default:
        return mainContent;
    }
  }

  postProcess(svg: string, tagMapping: string[]) {
    if (!tagMapping.length) {
      return svg;
    }
    const div = document.createElement('div');
    div.innerHTML = svg;
    const svgElem = div.firstElementChild;
    if (!svgElem) {
      return svg;
    }

    this.postProcessElement(svgElem, tagMapping);
    return div.innerHTML;
  }

  postProcessElement(elem: Element, tagMapping: string[]) {
    if (elem.tagName === 'path') {
      // console.log('found path', elem);
      // if (elem.)
      // path data starts with M 0 0 M
      const pathData = elem.getAttribute('d');
      if (!pathData) {
        return;
      }
      if (pathData.startsWith('M 0 0 M')) {
        elem.setAttribute('d', pathData.replace('M 0 0', '').trim());
      }
    }

    if (elem.tagName === 'g') {
      let pathChildren = elem;
      while (pathChildren && pathChildren.children.length === 1) {
        pathChildren = pathChildren.children[0];
      }
      const strokeWith = Number.parseFloat(pathChildren.getAttribute('stroke-width') || '0');
      if (Math.abs(strokeWith - 0.00012345) < 1e-6) {
        const color = pathChildren.getAttribute('stroke');
        // console.log('found color', color);
        if (!color) {
          return;
        }
        const tagIdx = Number.parseInt(color.replace('#', ''), 16);
        return tagMapping[tagIdx - 1];
      }
    }

    let elements = [];
    const nestElements = [];
    let tagStart: string | undefined = undefined;
    for (const child of elem.children) {
      let tag = this.postProcessElement(child, tagMapping);
      if (!tag) {
        elements.push(child);
        continue;
      }

      // console.log('found tagIdx', tag, tagStart);

      if (!tagStart) {
        tagStart = tag;
        if (elements.length) {
          nestElements.push([undefined, elements]);
          elements = [];
        }
      } else {
        // console.log('found', tagStart, tag);
        if (tag !== tagStart) {
          return;
        }
        nestElements.push([tag, elements]);
        elements = [];
        tagStart = undefined;
      }
    }

    if (elements.length === elem.children.length) {
      return;
    }

    // remove all children
    while (elem.firstChild) {
      elem.removeChild(elem.firstChild);
    }
    for (const [tag, elements] of nestElements) {
      if (!tag) {
        elem.append(...elements!);
        continue;
      }

      const g = document.createElement('g');
      g.setAttribute('id', `cetz-app-${tag}`);
      g.setAttribute('class', 'typst-cetz-elem');
      g.append(...elements!);
      elem.appendChild(g);
    }

    if (elements.length) {
      elem.append(...elements!);
    }

    return undefined;
  }

  async drawDefinition(def: ElementDefinition, extraArgs?: string) {
    let arg = extraArgs ?? '';
    const mainContent = `
#import "@preview/cetz:0.1.2"
#set page(margin: 0pt, width: auto, height: auto)
#let debug-label(_) = ()
#cetz.canvas({
  import cetz.draw: *
  ${this.sigs().join(INEW_LINE)}
  ${def.id}(${arg})
}, length: 1pt)
`;
    console.log({ mainContent });
    const content = await $typst.svg({ mainContent });
    return content;
  }

  sigs(interactive?: boolean) {
    const sigs: string[] = [];
    this.definitions.forEach(def => {
      if (interactive) {
        sigs.push(def.interactiveSig);
      } else {
        sigs.push(def.sig);
      }
    });
    // console.log(sigs);
    return sigs;
  }

  flushDefinitions(editor: EditorState) {
    // console.log('flushDefinitions', editor);
    if (editor.definitions) {
      this.definitions = new Map();
      for (const def of editor.definitions) {
        const props = Object.keys(def.props || {})
          .map(k => `${k}: ${def.props![k]}`)
          .join(', ');
        let draw = def.draw.trim();
        const sig = `let ${def.id}(${props}, tag: black, node-label: none) = ${draw}`;
        const interactiveSig =
          def.interactive !== false
            ? `let ${def.id}(${props}, tag: black, node-label: none) = {
  rect((0, 0), (1, 1), stroke: 0.00012345pt + tag)
  let debug-label(pos) = content(pos, box(fill: color.linear-rgb(153, 199, 240, 70%), inset: 5pt, [
    #set text(fill: color.linear-rgb(0, 0, 0, 70%))
    #node-label
  ]))
  ${draw}
  rect((0, 0), (1, 1), stroke: 0.00012345pt + tag)
}`
            : sig;
        // console.log({ sig, draw });
        this.definitions.set(def.id, { ...def, sig, interactiveSig });
      }
    }
    if (editor.width) {
      this.width = Number.parseFloat(editor.width.replace('pt', ''));
    }
    if (editor.height) {
      this.height = Number.parseFloat(editor.height.replace('pt', ''));
    }
    this.flushPreview();
  }

  flushMain(editor: any[]) {
    const main: any = {};
    let idx = 0;
    for (let i = 0; i < editor.length; i++) {
      const ins = editor[i];
      idx += 1;
      ins.idx = idx;
      main[ins.name] = ins;
    }
    this.main = main;
  }

  previewDefinition(id: string) {
    this.previewingDef = id;
    this.flushPreview();
  }

  insertElem(ty: string, id: string) {
    const def = this.definitions.get(ty);
    if (!def) {
      return;
    }
    this.main[id] = {
      type: ty,
      pos: [0, 0],
      name: id,
      idx: Object.keys(this.main).length,
    };
    this.flushPreview();
    this.syncMainContent();
  }
}

(window as any).$preview = new PreviewState();
