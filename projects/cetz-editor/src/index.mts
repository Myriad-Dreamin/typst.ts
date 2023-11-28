import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';
import type { WebAssemblyModuleRef } from '@myriaddreamin/typst.ts/dist/esm/wasm.mjs';

type ModuleSource = 'local' | 'jsdelivr';

/// Begin of Retrieve Wasm Modules from somewhere
/// We need a compiler module and a renderer module
/// - `@myriaddreamin/typst-ts-web-compiler`
/// - `@myriaddreamin/typst-ts-renderer`

// Bundle
// @ts-ignore
// import compiler from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
// @ts-ignore
// import renderer from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';

// window.$typst$moduleSource = 'local';

let moduleSource: ModuleSource = (window.$typst$moduleSource || 'jsdelivr') as any;

let compiler: WebAssemblyModuleRef;
let renderer: WebAssemblyModuleRef;

switch (moduleSource) {
  case 'jsdelivr':
    compiler = fetch(
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler@0.4.1-rc3/pkg/typst_ts_web_compiler_bg.wasm',
    );
    renderer = fetch(
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer@0.4.1-rc3/pkg/typst_ts_renderer_bg.wasm',
    );
    break;
  case 'local':
    compiler = fetch(
      'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
    );
    renderer = fetch(
      'http://127.0.0.1:20810/base/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
    );
    break;
  default:
    console.warn('unknown module source for importing typst module', moduleSource);
}

$typst.setCompilerInitOptions({
  getModule: () => compiler || window.$wasm$typst_compiler,
});
$typst.setRendererInitOptions({
  getModule: () => renderer || window.$wasm$typst_renderer,
});

/// End of Retrieve Wasm Modules from somewhere

/**
 * Newline with 2 spaces indent
 */
const INEW_LINE = '\n  ';

interface ElementDefinition {
  id: string;
  interactive?: boolean;
  props?: Record<string, any>;
  draw: string;

  sig: string;
  interactiveSig: string;
}

interface DefinitionEditorState {
  content?: string;
  width?: string;
  height?: string;
}

type ExportKind = 'svg' | 'pdf' | 'cetz';

/**
 * Global singleton state for preview
 */
export class PreviewState {
  /// Editor states
  /**
   * Map from definition id to definition
   */
  definitions: Map<string, ElementDefinition> = new Map();
  /**
   * Map from instance id to instance
   */
  instances: Record<string, any> = {};
  /**
   * Selected definition id for preview
   * @default "main"
   */
  previewingDef: string = '';
  /**
   * Preview panel element
   */
  panelElem: HTMLElement | null = null;
  /**
   * svg width of the preview panel
   */
  width: number = 500;
  /**
   * svg height of the preview panel
   */
  height: number = 500;
  /**
   * Update main content of the editor
   */
  updateMainContent: (content: string) => void = undefined!;

  /// Rendering states
  /**
   * Whether the preview is rendering main content
   */
  isMain = false;
  /**
   * Current svg element
   */
  svgElem: SVGSVGElement | null = null;
  /**
   * Selected element for drag
   */
  selectedElem: SVGElement | null = null;
  /**
   * Offset for drag
   */
  offset: any;
  /**
   * Transform for drag
   */
  transform: any;

  constructor() {}

  /**
   * Bind preview panel element
   * @param panel preview panel element
   */
  bindElement(panel: HTMLElement, updateMainContent: (content: string) => void) {
    this.panelElem = panel;
    this.updateMainContent = updateMainContent;

    // element.addEventListener('click', e => {});
    panel.addEventListener('mousedown', e => this.startDrag(e));
    panel.addEventListener('mousemove', e => this.drag(e));
    panel.addEventListener('mouseup', e => this.endDrag(e));
    panel.addEventListener('mouseleave', e => this.endDrag(e));
    panel.addEventListener('contextmenu', e => this.doToggleContextMenu(e));

    this.renderPreview();
  }

  /**
   * Get stored definition names of the preview
   * @returns definition names
   */
  getDefinitionNames() {
    return Array.from(this.definitions.keys());
  }

  getSignatures(interactive?: boolean) {
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

  /// Begin of Preview Actions

  /**
   * Select definition for preview
   * @param id definition id
   */
  doSelectDef(id: string) {
    this.previewingDef = id;
    this.renderPreview();
  }

  /**
   * Set definitions for preview
   * @param editor editor state
   */
  doSetDefinitions(editor: DefinitionEditorState) {
    // console.log('doSetDefinitions', editor);
    const defs = this.parseCode(editor.content ?? '', 'definition');
    if (defs) {
      this.definitions = new Map();
      for (const def of defs) {
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
    this.renderPreview();
  }

  /**
   * Set content for preview
   * @param editor editor state
   */
  doSetMainContent(editor: { content: string }) {
    const editorContent = this.parseCode(editor.content, 'main');

    const main: any = {};
    let idx = 0;
    for (let i = 0; i < editorContent.length; i++) {
      const ins = editorContent[i];
      idx += 1;
      ins.idx = idx;
      main[ins.name] = ins;
    }
    this.instances = main;
    this.renderPreview();
  }

  /**
   * Insert element to main content
   * @param ty definition id
   * @param id instance id
   * @returns
   */
  doInsertElem(ty: string, id: string) {
    const def = this.definitions.get(ty);
    if (!def) {
      return;
    }

    if (this.instances[id]) {
      return;
    }

    this.instances[id] = {
      type: ty,
      pos: [0, 0],
      name: id,
      idx: Object.keys(this.instances).length,
    };
    this.renderPreview();
    this.syncMainContent();
  }

  /**
   * Export as kind
   * See {@link ExportKind}
   * @param kind export kind
   */
  async doExport(kind: ExportKind) {
    let blobType: string;
    switch (kind) {
      case 'svg':
        blobType = 'image/svg+xml';
        break;
      case 'pdf':
        blobType = 'application/pdf';
        break;
      case 'cetz':
        blobType = 'text/plain';
        break;
      default:
        throw new Error(`unknown export kind ${kind}`);
    }

    const d = await this.exportAs(kind);
    var b = new Blob([d], { type: blobType });
    this.exportBlobTo(b);
  }

  /**
   * Toggle context menu for interactive elements
   */
  doToggleContextMenu(e: MouseEvent) {
    let target = this.findTaggedTypstElement(e.target as Element);
    if (!target) {
      return;
    }

    e.preventDefault();

    console.log('checkClickAction', target.id);

    const typstId = target.id.replace('cetz-app-', '');
    const ins = this.instances[typstId];
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
        this.renderPreview();
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

  /// End of Preview Actions

  /// Begin of Export Actions

  /**
   * Export blob to file
   * @param blob blob to export
   */
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

  /**
   * Export definition as cetz code
   * @param def definition
   * @param extraElemArgs element arguments
   * @returns
   */
  async exportDefinition(def: ElementDefinition, extraElemArgs?: string) {
    let elemArgs = `node-label: "t"`;
    if (extraElemArgs) {
      elemArgs += `, ${extraElemArgs}`;
    }
    const mainContent = `
#import "@preview/cetz:0.1.2"
#set page(margin: 3pt, width: auto, height: auto)
#let debug-label(_) = ()
#cetz.canvas({
  import cetz.draw: *
  ${this.getSignatures().join(INEW_LINE)}
  ${def.id}(${elemArgs})
}, length: 1pt)
`;
    console.log({ mainContent });
    const content = await this.previewSvg(mainContent);
    return content;
  }

  /**
   * Export as kind
   * @param kind kind of export
   * @param interactive whether to export as interactive cetz code
   */
  async exportAs(kind: 'cetz', interactive?: boolean): Promise<string>;
  async exportAs(kind: 'svg', interactive?: boolean): Promise<string>;
  async exportAs(kind: 'pdf', interactive?: boolean): Promise<Uint8Array>;
  async exportAs(kind: ExportKind, interactive?: boolean): Promise<string | Uint8Array>;
  async exportAs(kind: ExportKind, interactive?: boolean): Promise<string | Uint8Array> {
    const instances: string[] = [];
    let prevPos = [0, 0];
    let t = 1;
    const tagMapping: string[] = [];

    for (const [k, ins] of Object.entries(this.instances).sort((x, y) => {
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
  ${this.getSignatures(interactive).join(INEW_LINE)}
  ${instances.join(INEW_LINE)}
}, length: 1pt)
`;
    // console.log({ mainContent });

    switch (kind) {
      case 'svg':
        return postProcess(await this.previewSvg(mainContent));
      case 'pdf':
        return await this.previewPdf(mainContent);
      case 'cetz':
      default:
        return mainContent;
    }

    /**
     * Post process svg with instrumented tags
     * @param svgContent svg content
     * @returns post processed svg content
     */
    function postProcess(svgContent: string): string {
      if (!tagMapping.length) {
        /// No tag mapping, return original svg content
        return svgContent;
      }

      /// Parse svg content
      const svgDiv = document.createElement('div');
      svgDiv.innerHTML = svgContent;
      const svgElem = svgDiv.firstElementChild;
      if (!svgElem) {
        return svgContent;
      }

      /// Post process svg element
      postProcessElement(svgElem);

      /// Return post processed svg content
      return svgDiv.innerHTML;
    }

    /**
     * Post process svg element
     * @returns if the element is an instrumented tag, return the tag name,
     * otherwise return undefined
     */
    function postProcessElement(elem: Element) {
      /// Post process path element
      if (elem.tagName === 'path') {
        // console.log('found path', elem);

        /// Process `<path d="M 0 0 M ...">` to `<path d="M ...">`
        /// This would fix the bounding box of the path
        const pathData = elem.getAttribute('d');
        if (!pathData) {
          return;
        }
        /// path data starts with M 0 0 M
        if (pathData.startsWith('M 0 0 M')) {
          elem.setAttribute('d', pathData.slice('M 0 0 '.length));
        }

        return undefined;
      }

      /// Post process other elements

      /// Detect an instrumented tag
      if (elem.tagName === 'g') {
        /// Cast a group element to a single inner path element
        let pathChildren = elem;
        while (pathChildren && pathChildren.children.length === 1) {
          pathChildren = pathChildren.children[0];
        }

        const strokeWith = Number.parseFloat(pathChildren.getAttribute('stroke-width') || '0');
        if (Math.abs(strokeWith - 0.00012345) < 1e-8) {
          const color = pathChildren.getAttribute('stroke');
          // console.log('found color', color);
          if (!color) {
            return;
          }
          const tagIdx = Number.parseInt(color.replace('#', ''), 16);

          /// Return the tag name
          return tagMapping[tagIdx - 1];
        }
      }

      /// Post process children

      /// Check tags in children
      /**
       * Nested identified elements with tags
       * @example
       * ```typescript
       * [['c1', [elem1,elem2]], [undefined, [elem3]]]
       * ```
       */
      const nestElements: [string | undefined, Element[]][] = [];
      /**
       * Scanned elements to be appended to `nestElements`
       */
      let elements: Element[] = [];
      /**
       * Scanned tag start
       */
      let tagStart: string | undefined = undefined;
      for (const child of elem.children) {
        let tag = postProcessElement(child);
        if (!tag) {
          /// Not an instrumented tag, append to `elements`
          elements.push(child);
          continue;
        }

        // console.log('found tagIdx', tag, tagStart);

        /// Found an instrumented tag

        if (!tagStart) {
          /// No tag start, set tag start
          tagStart = tag;

          /// Account for untagged elements
          if (elements.length) {
            nestElements.push([undefined, elements]);
            elements = [];
          }
        } else {
          // console.log('found', tagStart, tag);

          /// Broken tag, reset tag start
          if (tag !== tagStart) {
            console.warn('broken tag', tagStart, tag);
            return;
          }

          /// Account for tagged elements
          nestElements.push([tag, elements]);
          elements = [];

          /// Reset tag start
          tagStart = undefined;
        }
      }

      /// No instrumented tag found, return directly
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

        /// Create a group element for the tag
        const g = document.createElement('g');
        g.setAttribute('id', `cetz-app-${tag}`);
        g.setAttribute('class', 'typst-cetz-elem');
        g.append(...elements!);
        elem.appendChild(g);
      }

      /// Append tail elements
      if (elements.length) {
        elem.append(...elements!);
      }

      return undefined;
    }
  }

  async previewSvg(mainContent: string): Promise<string> {
    await $typst.addSource('/preview.typ', mainContent);
    return $typst.svg({ mainFilePath: '/preview.typ' });
  }

  async previewPdf(mainContent: string): Promise<Uint8Array> {
    await $typst.addSource('/preview.typ', mainContent);
    return $typst.pdf({ mainFilePath: '/preview.typ' });
  }

  /// End of Export Actions
  /// Begin of DOM State Fetch/Push Actions

  /**
   * A Fetch Action from DOM
   *
   * Find the tagged typst element from the target element
   * @param target
   * @returns
   */
  findTaggedTypstElement(target: Element) {
    while (target) {
      if (target.classList.contains('typst-cetz-elem')) {
        return target;
      }
      target = target.parentElement!;
    }
    return undefined;
  }

  /**
   * A Fetch Action from DOM
   *
   */
  getMousePosition(evt: MouseEvent) {
    var CTM = this.svgElem!.getScreenCTM()!;
    return {
      x: (evt.clientX - CTM.e) / CTM.a,
      y: (evt.clientY - CTM.f) / CTM.d,
    };
  }

  /**
   * A Push Action to DOM
   *
   */
  syncMainContent() {
    const data: string[] = [];
    for (const [_, ins] of Object.entries(this.instances).sort((x, y) => {
      return x[1].idx - y[1].idx;
    })) {
      let args = '';
      const argEntries = Object.entries(ins.args ?? {});
      if (argEntries.length) {
        for (const [k, v] of argEntries) {
          if (args) {
            args += ', ';
          }
          args += `${k}: ${v}`;
        }
      }
      let [x, y] = ins.pos;
      if (ins.deltaPos) {
        x = ins.initPos[0] + ins.deltaPos[0];
        y = ins.initPos[1] - ins.deltaPos[1];
      }
      // data.push(`- name: ${ins.name}\n  type: ${ins.type}${args}\n  pos:
      // [${x}, ${y}]`);
      x = Math.round(x * 1000) / 1000;
      y = Math.round(y * 1000) / 1000;
      data.push(`make-ins("${ins.name}", (${x}, ${y}), "${ins.type}", (${args}))`);
    }
    this.updateMainContent(`#{
  ${data.join(INEW_LINE)}
}`);
  }

  /// End of Render State Fetch/Push Actions

  /// Begin of Rendering Actions

  previewPromise: Promise<void> | null = null;

  /**
   * Flush Rendering preview panel
   */
  renderPreview() {
    if (this.previewPromise) {
      this.previewPromise = this.previewPromise
        .then(() => this.workPreview())
        .catch(e => console.log(e));
    } else {
      this.previewPromise = this.workPreview();
    }
  }

  /**
   * Work for rendering preview panel
   */
  async workPreview() {
    if (!this.panelElem) {
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
      content = await this.exportAs('svg', true);
      isMain = () => true;
    } else {
      const def = this.definitions.get(this.previewingDef);
      if (def) {
        // console.log('previewing definition', def, $typst, this);
        content = await this.exportDefinition(def);
      }
    }

    if (content !== undefined) {
      // console.log({ content });
      this.panelElem.innerHTML = content;

      const svgElem = this.panelElem.firstElementChild;
      this.svgElem = svgElem as any;
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

  /// End of Rendering Actions

  /// Begin of Rendering Drag Actions

  startDrag(evt: MouseEvent) {
    if (!this.isMain) {
      return;
    }

    let target = this.findTaggedTypstElement(evt.target as Element);
    if (target) {
      const elem = (this.selectedElem = target as any);

      this.offset = this.getMousePosition(evt);
      // Get all the transforms currently on this element
      let transforms = elem.transform.baseVal;
      // Ensure the first transform is a translate transform
      if (
        transforms.length === 0 ||
        transforms.getItem(0).type !== SVGTransform.SVG_TRANSFORM_TRANSLATE
      ) {
        // Create an transform that translates by (0, 0)
        var translate = this.svgElem!.createSVGTransform();
        translate.setTranslate(0, 0);
        // Add the translation to the front of the transforms list
        elem.transform.baseVal.insertItemBefore(translate, 0);
      }
      // Get initial translation amount
      this.transform = transforms.getItem(0);
      this.offset.x -= this.transform.matrix.e;
      this.offset.y -= this.transform.matrix.f;

      const typstId = target.id.replace('cetz-app-', '');
      if (!this.instances[typstId].initPos) {
        this.instances[typstId].initPos = [...this.instances[typstId].pos];
      }
    } else {
      this.selectedElem = null;
    }
  }

  drag(evt: MouseEvent) {
    const selectedElement = this.selectedElem;
    if (selectedElement) {
      evt.preventDefault();
      var coord = this.getMousePosition(evt);
      const x = coord.x - this.offset.x;
      const y = coord.y - this.offset.y;
      this.transform.setTranslate(x, y);
      const typstId = selectedElement.id.replace('cetz-app-', '');
      this.instances[typstId].deltaPos = [x, y];
      this.syncMainContent();
    }
  }

  endDrag(_evt: MouseEvent) {
    if (this.selectedElem) {
      const typstId = this.selectedElem.id.replace('cetz-app-', '');
      const ins = this.instances[typstId];
      if (ins.deltaPos) {
        ins.pos[0] = ins.initPos[0] + ins.deltaPos[0];
        ins.pos[1] = ins.initPos[1] - ins.deltaPos[1];
        ins.deltaPos = undefined;
        ins.initPos = undefined;
        console.log(JSON.stringify(ins));
        setTimeout(() => {
          this.renderPreview();
        }, 16);
      }
      this.selectedElem = null;
    }
  }

  /// End of Rendering Drag Actions

  parseCode(s: string, mode: 'definition'): ElementDefinition[];
  parseCode(s: string, mode: 'main'): any[];
  parseCode(s: string, mode: 'main' | 'definition') {
    // ElementDefinition[]
    // console.log('parseCode', s);

    // analyze brackets
    let leftChars: [number, string][] = [];
    let jumpMap = new Map<number, number>();
    for (let i = 0; i < s.length; i++) {
      const c = s[i];
      if (c === '(' || c === '{' || c === '[') {
        leftChars.push([i, c]);
      } else if (c === ')' || c === '}' || c === ']') {
        const left = leftChars.pop();
        if (left === undefined) {
          throw new Error(`unmatched right bracket at ${i}`);
        }
        switch (c) {
          case ')':
            if (left[1] !== '(') {
              throw new Error(`unmatched right bracket at ${i}`);
            }
            break;
          case '}':
            if (left[1] !== '{') {
              throw new Error(`unmatched right bracket at ${i}`);
            }
            break;
          case ']':
            if (left[1] !== '[') {
              throw new Error(`unmatched right bracket at ${i}`);
            }
            break;
        }
        jumpMap.set(left[0], i);
      }
    }

    // console.log(jumpMap);

    // analyze definitions

    function skipLine(i: number) {
      for (; i < s.length; i++) {
        const c = s[i];
        if (c === '\n') {
          return i;
        }
      }
      return i;
    }

    function skipSpaces(i: number) {
      for (; i < s.length; i++) {
        const c = s[i];
        if (!' \t\r\n'.includes(c)) {
          return i;
        }
      }
      return i;
    }

    const identMatcher = /^[a-zA-Z_][a-zA-Z0-9_\-]*/;
    function skipIdent(i: number) {
      let res = identMatcher.exec(s.slice(i));
      // console.log(res, s.slice(i));
      if (!res) {
        return i;
      }

      return i + res[0].length;
    }

    const valueMatcher =
      /^(?:((?:0x[a-zA-Z_]+)|(?:0[a-zA-Z\d]+)|(?:\d*\.?\d*E\-?\d*)|(?:\d*\.(?:\d{0,3}_?)*)|(?:\d+)|(?:\d*\.\d+)))|(?:["`]([^"`\\]|\\.)*["`])/;
    function skipValue(i: number) {
      let res = valueMatcher.exec(s.slice(i));
      // console.log(res, s.slice(i));
      if (!res) {
        return i;
      }

      return i + res[0].length;
    }

    function skipExpression(i: number) {
      if ('([{'.includes(s[i])) {
        let bodyRight = jumpMap.get(i);
        if (bodyRight === undefined) {
          throw new Error(`unmatched left bracket at ${i}`);
        }

        return bodyRight + 1;
      } else {
        let j = skipIdent(i);
        if (i !== j) {
          return j;
        }

        j = skipValue(i);
        if (i !== j) {
          return j;
        }

        return skipLine(i);
      }
    }

    function eatToken(i: number, token: string) {
      if (s[i] !== token) {
        throw new Error(`expected token ${token} at ${i}`);
      }
      return i + 1;
    }

    function parseProps([argsLeft, argsRight]: number[]) {
      // console.log('parseArgs', s.slice(argsLeft, argsRight));

      const props: Record<string, any> = {};

      for (;;) {
        let i = skipSpaces(argsLeft);
        if (i === argsRight) {
          break;
        }

        let j = skipIdent(i);
        if (i === j) {
          throw new Error(`expected ident at ${i} ${s.slice(i)}`);
        }

        const ident = s.slice(i, j);
        i = skipSpaces(eatToken(skipIdent(j), ':'));
        // console.log('found prop', ident, '<<', s.slice(i, i + 10));

        let valueRight = skipExpression(i);
        let defaultValue = s.slice(i, valueRight);
        i = skipSpaces(valueRight);
        // console.log('found propDefault', defaultValue, '<<', s.slice(i, i + 10));

        props[ident] = defaultValue;

        if (s[i] === ',') {
          i += 1;
        }

        if (i <= argsLeft) {
          break;
        }

        argsLeft = i;
        // console.log('next round <<', s.slice(i, i + 10));
      }

      return props;
    }

    function parseDefinition() {
      const defs: ElementDefinition[] = [];
      for (let i = 0; i < s.length; i++) {
        const c = s[i];
        if (' \t\r\n'.includes(c)) {
          continue;
        }
        if (c === '#') {
          i += 1;
          // console.log('found #', i, s.slice(i, i + 10));

          if (!s.slice(i).startsWith('let')) {
            i = skipLine(i);
            continue;
          }

          i += 3;
          i = skipSpaces(i);
          // console.log('found #let', i, '<<', s.slice(i, i + 10));

          let j = skipIdent(i);
          let ident = s.slice(i, j);
          // console.log('found #let ident', s.slice(i, j), '<<', s.slice(j, j + 10));
          i = j;

          i = skipSpaces(i);

          if (!s.slice(i).startsWith('(')) {
            i = skipLine(i);
            continue;
          }

          let fnArgsRight = jumpMap.get(i);
          if (fnArgsRight === undefined) {
            throw new Error(`unmatched left bracket at ${i}`);
          }

          let fnArgs = [i + 1, fnArgsRight];
          i = fnArgsRight + 1;

          // console.log('found #let ident(args = ...)', s.slice(i, i + 10));

          i = skipSpaces(eatToken(skipSpaces(i), '='));

          // console.log('found #let ident(args = ...) = $exp = ', s.slice(i, i + 10));

          let bodyRight = skipExpression(i);
          let body = s.slice(i, bodyRight);
          i = bodyRight;

          const def = {
            id: ident,
            props: parseProps(fnArgs),
            draw: body,
          };
          // console.log('found definition', def);

          defs.push(def as ElementDefinition);
        } else if (c === '/') {
          i += 1;
          if (s.slice(i).startsWith('/')) {
            i = skipLine(i);
          } else if (s.slice(i).startsWith('*')) {
            i += 1;
            while (i < s.length) {
              if (s.slice(i).startsWith('*/')) {
                i += 2;
                break;
              }
              i += 1;
            }
          } else {
            throw new Error(`invalid character ${c} at ${i}`);
          }
        } else {
          throw new Error(`invalid character ${c} at ${i}`);
        }
      }

      return defs;
    }

    function parseMainContent() {
      console.log('parseMainContent', s);
      const instances: any[] = [];
      let i = 0;
      i = eatToken(skipSpaces(i), '#');
      if (!s.slice(i).startsWith('{')) {
        throw new Error(`expected left bracket at ${i}`);
      }

      let j = jumpMap.get(i);
      if (j === undefined) {
        throw new Error(`unmatched left bracket at ${i}`);
      }
      let bodyRight = j;
      j = skipSpaces(j + 1);
      if (j !== s.length) {
        throw new Error(`unexpected content at ${j}`);
      }
      // console.log('found #{ ... }', s.slice(i + 1, j - 1));

      let bodyLeft = i + 1;
      while (bodyLeft < bodyRight) {
        i = bodyLeft;
        i = skipSpaces(i);
        if (i >= bodyRight) {
          break;
        }
        j = skipIdent(i);
        if (i === j) {
          throw new Error(`expected ident at ${i} ${s.slice(i)}`);
        }
        let ident = s.slice(i, j);
        i = skipSpaces(j);
        // console.log('found ident', ident, '<<', s.slice(i, i + 10));

        if (!s.slice(i).startsWith('(')) {
          throw new Error(`expected call at ${i}`);
        }
        let fnArgsRight = jumpMap.get(i);
        if (fnArgsRight === undefined) {
          throw new Error(`unmatched left bracket at ${i}`);
        }
        let fnArgs = [];

        i += 1;
        while (i < fnArgsRight) {
          i = skipSpaces(i);
          let arg = skipExpression(i);
          fnArgs.push([i, arg]);
          i = skipSpaces(arg);
          if (s[i] === ',') {
            i += 1;
          }
        }

        i = fnArgsRight + 1;

        // console.log(
        //   'found call',
        //   { ident, fnArgs: fnArgs.map(e => s.slice(e[0], e[1])) },
        //   i,
        //   '<<',
        //   s.slice(i, i + 10),
        // );

        if (ident === 'make-ins') {
          fnArgs = fnArgs.map(e => [e[0] + 1, e[1] - 1]);
          const nodeName = s.slice(fnArgs[0][0], fnArgs[0][1]);
          const position = s
            .slice(fnArgs[1][0], fnArgs[1][1])
            .split(',')
            .map(e => Number.parseFloat(e.trim()));
          const ty = s.slice(fnArgs[2][0], fnArgs[2][1]);
          // console.log('parseProps', s.slice(fnArgs[3][0], fnArgs[3][1]));
          const fnArgsParsed = parseProps(fnArgs[3]);
          const ins = {
            name: nodeName,
            pos: position,
            type: ty,
            args: fnArgsParsed,
          };
          // console.log('found instance', ins);
          instances.push(ins);
        } else {
          throw new Error(`unknown function ${ident} at ${i}`);
        }

        if (i <= bodyLeft) {
          break;
        }
        bodyLeft = i;
      }

      return instances;
    }

    switch (mode) {
      case 'definition':
        return parseDefinition();
      case 'main':
        return parseMainContent();
      default:
        throw new Error(`unknown mode ${mode}`);
    }
  }

  async queryInstances(content: string) {
    const instances: any[] = [];
    await $typst.addSource(
      '/cetz-editor-preset.typ',
      `#let instances-state = state("instance", ())
#let template(content) = {
  locate(loc => [
    #metadata(instances-state.final(loc)) <cetz-instances>
  ])
  content
}
#let make-ins(name, pos, factory, props) = {
  instances-state.update(it => {
    it.push((name: name, pos: pos, factory: factory, props: props))
    it
  })
}`,
    );
    await $typst.addSource('/query-instance.typ', content);
    const checkedInstances = await $typst.query({
      mainFilePath: '/query-instance.typ',
      selector: '<cetz-instances>',
    });
    console.log(checkedInstances);
    return instances;
  }
}

const scopeRenamingRules: { scopes: Record<string, any> } = {
  scopes: {
    // vscode rules
    namespace: ['entity.name.namespace'],
    type: ['entity.name.type'],
    'type.defaultLibrary': ['support.type'],
    struct: ['storage.type.struct'],
    class: ['entity.name.type.class'],
    'class.defaultLibrary': ['support.class'],
    interface: ['entity.name.type.interface'],
    enum: ['entity.name.type.enum'],
    function: ['entity.name.function'],
    'function.defaultLibrary': ['support.function'],
    method: ['entity.name.function.member'],
    macro: ['entity.name.function.macro'],
    variable: ['variable.other.readwrite , entity.name.variable'],
    'variable.readonly': ['variable.other.constant'],
    'variable.readonly.defaultLibrary': ['support.constant'],
    parameter: ['variable.parameter'],
    property: ['variable.other.property'],
    'property.readonly': ['variable.other.constant.property'],
    enumMember: ['variable.other.enummember'],
    event: ['variable.other.event'],

    // typst rules
    '*.strong.emph': ['markup.bold.typst markup.italic.typst'],
    '*.strong': ['markup.bold.typst'],
    '*.emph': ['markup.italic.typst'],
    '*.math': ['markup.math.typst'],
    bool: ['constant.language.boolean.typst'],
    punct: ['punctuation.typst', 'punctuation.definition.typst'],
    escape: [
      'constant.character.escape.typst',
      'keyword.operator.typst',
      'punctuation.definition.typst',
    ],
    link: ['markup.underline.link.typst'],
    raw: ['markup.inline.raw.typst', 'markup.raw.inline.typst'],
    'delim.math': [
      'punctuation.definition.math.typst',
      'punctuation.definition.string.end.math.typst',
      'string.quoted.other.typst',
    ],
    pol: ['variable.other.readwrite , entity.name.variable'],
  },
};

const typstTokens = [
  'comment',
  'string',
  'keyword',
  'operator',
  'number',
  'function',
  'decorator',
  'bool',
  'punctuation',
  'escape',
  'link',
  'raw',
  'label',
  'ref',
  'heading',
  'marker',
  'term',
  'delim',
  'pol',
  'error',
  'text',
];

(window as any).adaptVsCodeThemeForTypst = (theme: any) => {
  const { tokenColors, semanticTokenColors } = theme;

  // ...Object.keys(vscodeTheme.semanticTokenColors).map((k) => {
  //   return { token: k, foreground: vscodeTheme.semanticTokenColors[k] };
  // }),
  const newTokenColors: any[] = [];
  const defaultSettings = { foreground: theme.colors.foreground };

  const rules = new Map<string, any>();
  for (const tokenColor of tokenColors) {
    for (const s of tokenColor.scope) {
      rules.set(s, tokenColor);
    }
  }

  const semanticTokenRules = new Map<string, any>();
  for (const [k, settings] of Object.entries(semanticTokenColors)) {
    semanticTokenRules.set(k, settings);

    if (k.startsWith('*.')) {
      let suffix = k.slice('*.'.length);
      for (const token of typstTokens) {
        semanticTokenRules.set(`${token}.${suffix}`, settings);
      }
    }
  }

  for (const [k, renamedScopes] of Object.entries(scopeRenamingRules.scopes)) {
    let scopes: string[] = [];
    if (k.startsWith('*.')) {
      let suffix = k.slice('*.'.length);
      for (const token of typstTokens) {
        scopes.push(`${token}.${suffix}`);
      }
    } else {
      scopes.push(k);
    }

    let settings = defaultSettings;
    if (semanticTokenRules.has(k)) {
      settings = semanticTokenRules.get(k);
    } else {
      for (let i = 0; i < renamedScopes.length; i++) {
        const renamedScope = renamedScopes[i];

        for (let j = renamedScope.length; j > 0; j--) {
          if (j !== renamedScope.length && renamedScope[j] !== '.') {
            continue;
          }
          const rule = rules.get(renamedScope.slice(0, j));
          if (rule) {
            settings = rule.settings;
            break;
          }
        }
      }
    }
    newTokenColors.push({
      name: `typst ${k}`,
      scope: scopes,
      settings,
    });
  }

  // console.log('newTokenColors', rules, newTokenColors);
  return {
    ...theme,
    tokenColors: [...tokenColors, ...newTokenColors],
  };
};

class SemanticTokensProvider {
  constructor(private legend: any) {}

  getLegend() {
    return this.legend;
  }

  async provideDocumentSemanticTokens(model: any, lastResultId: string) {
    // todo: support incremental update
    void lastResultId;
    await $typst.addSource('/semantic-tokens.typ', model.getValue());
    return $typst.getSemanticTokens({ mainFilePath: '/semantic-tokens.typ' });
  }

  async releaseDocumentSemanticTokens(resultId: string) {}
}

window.$preview = new PreviewState();

window.$typst$semanticTokensProvider = $typst.getSemanticTokenLegend().then(legend => {
  return new SemanticTokensProvider(legend);
});
