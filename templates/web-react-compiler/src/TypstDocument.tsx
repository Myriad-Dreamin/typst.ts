import { useRef, useEffect, useState } from 'react';

import type * as typst from '@myriaddreamin/typst.ts';
import { createGlobalRenderer } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs';
import { createTypstRenderer } from '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';
import { withGlobalCompiler } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-compiler.mjs';
import { createTypstCompiler } from '@myriaddreamin/typst.ts/dist/esm/compiler.mjs';
import { preloadRemoteFonts } from '@myriaddreamin/typst.ts/dist/esm/options.init.mjs';
import compiler from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import renderer from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';

import htmlLayerCss from './typst.css?inline';

export interface TypstDocumentProps {
  /**
   * The background color of the document.
   */
  fill?: string;
  /**
   * The source code of the Typst document. Only one of `source` or `artifact` can be provided.
   */
  source?: string;
  /**
   * The artifact of the Typst document. Only one of `source` or `artifact` can be provided.
   */
  artifact?: Uint8Array;
  /**
   * A compiler handler. If not provided, a global *shared* compiler will be created.
   */
  compiler?: typst.TypstCompiler;
  /**
   * A renderer handler. If not provided, a global *shared* renderer will be created.
   */
  renderer?: typst.TypstRenderer;
  /**
   * A callback function to handle diagnostics. If not provided, the default `console.error` will be used.
   */
  onDiagnostics?: (diagnostics: unknown) => void;
}

let compilerInitOpts: typst.InitOptions = {
  beforeBuild: [preloadRemoteFonts([])],
  getModule: () => {
    console.log('compiler', compiler);
    return compiler;
  },
};

let rendererInitOpts: typst.InitOptions = {
  beforeBuild: [],
  getModule: () => renderer,
};

export const TypstDocument = ({
  fill,
  source,
  artifact,
  compiler,
  renderer,
  onDiagnostics,
}: TypstDocumentProps) => {
  if (source && artifact) {
    throw new Error('Cannot provide both source and artifact, please provide only one.');
  }

  /// Diag callback
  const setDiag = onDiagnostics ?? console.error;

  const displayDivRef = useRef<HTMLDivElement>(null);
  const getDisplayLayerDiv = () => {
    return displayDivRef?.current;
  };

  const [rHandler, setRenderer] = useState<RendererResource | undefined>();
  const [finalArtifact, setArtifact] = useState<Uint8Array | undefined>(artifact);

  /// Creates renderer and session
  useEffect(() => {
    let killPromiseCallback;
    const killPromise = new Promise(resolve => (killPromiseCallback = resolve));

    (async () => {
      /// Creates global renderer if no renderer is provided
      const r = renderer ?? (await createGlobalRenderer(createTypstRenderer, rendererInitOpts));
      /// Creates session
      return r.runWithSession(async session => {
        console.log('new session', session);
        const rHandler = { session, renderer: r };
        setRenderer(rHandler);
        await killPromise;
        console.log('session killed', session);
      });
    })();

    return killPromiseCallback!;
  }, [renderer]);

  /// Creates compiler, and compiles document
  useEffect(() => {
    if (!source) {
      return;
    }

    const doCompile = async (c: typst.TypstCompiler) => {
      compiler = c;
      if (!source) {
        return;
      }

      c.addSource('/main.typ', source);
      const result = await c.compile({
        mainFilePath: '/main.typ',
      });

      if (result.diagnostics) {
        if (setDiag) {
          setDiag(result.diagnostics);
        }
      } else {
        setArtifact(result.result);
      }
    };

    /// compile after init
    if (compiler) {
      doCompile(compiler);
    } else {
      withGlobalCompiler(createTypstCompiler, compilerInitOpts, doCompile);
    }
  }, [source, compiler]);

  /// Renders document
  useEffect(() => {
    /// get display layer div
    const divContainerElem = getDisplayLayerDiv();
    if (!divContainerElem) {
      return;
    }
    if (!divContainerElem.firstElementChild) {
      const wrapper = document.createElement('div');
      wrapper.className = 'display-layer-wrapper';
      divContainerElem.appendChild(wrapper);

      const div = document.createElement('div');
      wrapper.appendChild(div);
    }
    const wrapElem = divContainerElem.firstElementChild! as HTMLDivElement;
    const divElem = wrapElem.firstElementChild as HTMLDivElement;

    /// we allow empty artifact
    if (!finalArtifact?.length) {
      divElem.innerHTML = '';
      return;
    }

    /// render after init
    if (rHandler) {
      rHandler.session.manipulateData({
        action: 'merge',
        data: finalArtifact,
      });

      const docWidth = rHandler.session.docWidth;
      if (docWidth && docWidth > 0) {
        const dw = `${docWidth * window.devicePixelRatio}`;
        if (wrapElem.dataset.width !== dw) {
          wrapElem.dataset.width = dw;
          wrapElem.style.width = `calc(min(${dw}px, 100%))`;
        }
      }

      rHandler.renderer.renderToCanvas({
        renderSession: rHandler.session,
        format: 'vector',
        backgroundColor: fill,
        container: divElem,
        pixelPerPt: 3,
      });
    }
  }, [displayDivRef, fill, finalArtifact, rHandler]);

  /// --- end: update document --- ///

  return (
    <div>
      {/* todo: remove this embedded css */}
      <style>{htmlLayerCss}</style>
      <div className="typst-app" ref={displayDivRef}></div>
    </div>
  );
};

interface RendererResource {
  session: typst.RenderSession;
  renderer: typst.TypstRenderer;
}
