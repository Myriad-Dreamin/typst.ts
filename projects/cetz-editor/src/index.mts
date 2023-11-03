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
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
    );
    renderer = fetch(
      'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
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
  definitions?: ElementDefinition[];
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
    this.renderPreview();
  }

  /**
   * Set content for preview
   * @param editor editor state
   */
  doSetMainContent(editor: any[]) {
    const main: any = {};
    let idx = 0;
    for (let i = 0; i < editor.length; i++) {
      const ins = editor[i];
      idx += 1;
      ins.idx = idx;
      main[ins.name] = ins;
    }
    this.instances = main;
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
    const content = await $typst.svg({ mainContent });
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
        return postProcess(await $typst.svg({ mainContent }));
      case 'pdf':
        return await $typst.pdf({ mainContent });
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
}

window.$preview = new PreviewState();
