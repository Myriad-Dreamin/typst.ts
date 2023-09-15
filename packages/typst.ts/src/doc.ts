import { PageInfo, Rect, TransformMatrix } from './internal.types';
import { RenderOptions } from './options.render';
import { patchRoot } from './render/svg/patch';
import { RenderSession, TypstRenderer } from './renderer';

export interface LayoutContext {}

export interface PageView {
  pages: PageInfo[];
}

export interface RenderPiece {
  pageOffset?: number;
  invisible?: boolean;
  at?: Element;
  window?: Rect;
  ts?: TransformMatrix;
}

export interface TypstDocumentProps {
  plugin: TypstRenderer;
  shadowRoot: Element;
  // default: svg
  renderMode?: 'svg' | 'svg-group' | 'canvas';

  session?: RenderSession;

  layoutPages?(doc: TypstDocument, before: PageView): Promise<RenderPiece[]>;
}

/**
 * The options for manipulating the Typst document in the session.
 */
interface DocumentDataChangement {
  /**
   * The action to manipulate the data.
   * @description `reset-doc`: reset the data to the initial state.
   * @description `merge-doc`: merge the data to the current state.
   */
  action: 'reset-doc' | 'merge-doc';
  /**
   * Opaque data to manipulate the Typst document from server.
   */
  data: Uint8Array;
}

/**
 * The options for manipulating the Typst document in the session.
 */
interface DocumentViewportChangement {
  /**
   * Change the viewport of the Typst document.
   */
  action: 'viewport-change';
}

/**
 * The options for manipulating the Typst document in the session.
 */
export type DocumentChangement = DocumentDataChangement | DocumentViewportChangement;

export interface LayoutSvgPageOptions {
  /// wrap in svg-group mode
  wrapG?(g: [Rect, SVGGElement]): [Rect, SVGGElement];
}

export interface TypstDocument {
  shadowRoot: Element;
  session: RenderSession;
  onSessionReady(): Promise<RenderSession>;

  changeLayout(layoutSelector: Record<string, any>): void;

  addChangements(change: DocumentChangement[]): void;
  addViewportChange(): void;

  renderPieces(pieces: RenderPiece[]): Promise<void>;

  dispose(): void;
}

export class TypstDocument implements TypstDocument {
  session: RenderSession;
  private sessionReady: Promise<RenderSession>;
  private disposeSession: (...args: any[]) => any;

  constructor(private props: RenderOptions<TypstDocumentProps>) {
    this.init();
  }

  static layoutPagesFullMode(doc: TypstDocument): Promise<RenderPiece[]> {
    return Promise.resolve(
      doc.session.retrievePagesInfo().map(() => {
        return {};
      }),
    );
  }

  // copy from typst-preview
  static layoutSvgPagesPartialMode(doc: TypstDocument, before: PageView): Promise<RenderPiece[]> {
    const docRect = doc.shadowRoot.getBoundingClientRect(); // this.cachedBoundingRect;
    // https://measurethat.net/Benchmarks/Show/5392/0/clientwidth-vs-offsetwidth-vs-windowgetcomputedstyle
    // todo: this is only a PoC
    const cachedOffsetWidth: number =
      'offsetWidth' in (doc.shadowRoot as HTMLElement)
        ? (doc.shadowRoot as HTMLElement).offsetWidth
        : Number.parseInt(window.getComputedStyle(doc.shadowRoot).width.replace('px', ''));
    const currentScaleRatio = 1; // this.currentScaleRatio;
    // scale derived from svg width and container with.
    const computedRevScale = cachedOffsetWidth ? doc.session.docWidth / cachedOffsetWidth : 1;
    // respect current scale ratio
    const revScale = computedRevScale / currentScaleRatio;
    const left = (window.screenLeft - docRect.left) * revScale;
    const top = (window.screenTop - docRect.top) * revScale;
    const width = window.innerWidth * revScale;
    const height = window.innerHeight * revScale;

    void before;
    const patchStr = doc.session.renderSvgDiff({
      window: {
        lo: {
          x: left,
          y: top,
        },
        hi: {
          x: left + width,
          y: top + height,
        },
      },
    });
    // todo: ideally, we should patch per page separately
    // todo: then, we can call the api doc.renderPieces(pages + windows).

    if (doc.shadowRoot.firstElementChild) {
      const elem = document.createElement('div');
      elem.innerHTML = patchStr;
      const svgElement = elem.firstElementChild as SVGElement;
      patchRoot(doc.shadowRoot.firstElementChild as SVGElement, svgElement);
    } else {
      doc.shadowRoot.innerHTML = patchStr;
    }

    return Promise.resolve([]);
  }

  // copy from typst-preview
  // todo: generalize patchRoot here
  static layoutSvgPages(
    options: LayoutSvgPageOptions,
  ): (doc: TypstDocument, before: PageView) => Promise<RenderPiece[]> {
    return (doc: TypstDocument, before: PageView) => {
      // todo
      return TypstDocument.layoutSvgPagesPartialMode(doc, before);
    };
  }

  addChangements(change: DocumentChangement[]): void {
    throw new Error('Method not implemented.');
  }

  renderPieces(pieces: RenderPiece[]): Promise<void> {
    throw new Error('Method not implemented.');
  }

  onSessionReady(): Promise<RenderSession> {
    return this.sessionReady;
  }

  dispose() {
    if (this.disposeSession !== undefined) {
      this.disposeSession();
    }
  }

  private init(): Promise<RenderSession> {
    if (this.sessionReady !== undefined) {
      throw new Error('Already initialized');
    }

    if (this.props.session !== undefined) {
      this.session = this.props.session;
      return (this.sessionReady = Promise.resolve(this.session));
    }

    return (this.sessionReady = new Promise(resolve => {
      this.props.plugin.runWithSession(session => {
        return new Promise(dispose => {
          this.session = session;
          this.disposeSession = dispose;
          resolve(session);
        });
      });
    }));
  }
}

//   private collectDocProperties(): DocProperties {
//     return {
//       [kRects]: undefined,
//     };
//   }

//   async mutate(
//     cb: (ctx: PageMutationContext) => Promise<void>,
//     // options?: {
//     //   prefetchedDocProperties?: DocProperties;
//     // },
//   ): Promise<void> {
//     await this.sessionReady;

//     // const docProperties = options?.prefetchedDocProperties ?? this.collectDocProperties();
//     const ctx = new PageMutationContextImpl(this.props, docProperties);
//     await cb(ctx);
//     return ctx[kDispose]();
//   }

// import { ManipulateDataOptions } from '../../options.render';
// export interface PageMutationInst<K> {
//   /**
//    * The kind of mutation.
//    * @property {string} kind - The kind of mutation.
//    */
//   kind: K;

//   /**
//    * After/At which element the mutation happens.
//    */
//   sentinel?: Element;
// }

// export type PageMutation = PageMutationInst<'create' | 'delete' | 'change'>;
// const kRects = Symbol('rects');

// class PageMutationContextImpl implements PageMutationContext {
//   mutations: PageMutation[] = [];
//   constructor(
//     private props: TypstDocumentProps,
//     private docProperties: DocProperties,
//   ) {}

//   manipulateData(opts: ManipulateDataOptions) {
//     throw new Error('Method not implemented.');
//   }

//   //   mutateSeq(mutation: PageMutation[]): Promise<Element[]> {
//   //     return Promise.all(mutation.map(this.mutateOne));
//   //   }

//   //   mutateOne(mutation: PageMutation): Promise<Element> {
//   //     throw new Error('Method not implemented. ' + JSON.stringify(mutation));
//   //   }

//   [kDispose](): void {}
// }

// export interface DocProperties {
//   [kRects]: unknown;
// }
