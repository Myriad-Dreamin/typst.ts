import { useRef, useEffect, useState } from 'react';
import { createGlobalRenderer } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs';
import { createTypstRenderer } from '@myriaddreamin/typst.ts/dist/esm/renderer.mjs';
import { withGlobalCompiler } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-compiler.mjs';
import { createTypstCompiler } from '@myriaddreamin/typst.ts/dist/esm/compiler.mjs';
import { preloadRemoteFonts } from '@myriaddreamin/typst.ts/dist/esm/options.init.mjs';
import type * as typst from '@myriaddreamin/typst.ts';
import htmlLayerCss from './typst.css?inline';
import compiler from '@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm?url';
import renderer from '@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm?url';

export interface TypstDocumentProps {
  fill?: string;
  source?: string;
  artifact?: Uint8Array;
  // todo: add vector format
  format?: 'json';
  compiler?: typst.TypstCompiler;
  renderer?: typst.TypstRenderer;
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
  format,
  compiler,
  renderer,
  onDiagnostics,
}: TypstDocumentProps) => {
  if (source && artifact) {
    throw new Error('Both source and artifact are provided, please provide only one.');
  }
  const setDiag = onDiagnostics ?? console.error;

  /// --- beg: update document --- ///
  const displayDivRef = useRef<HTMLDivElement>(null);
  const getDisplayLayerDiv = () => {
    return displayDivRef?.current;
  };

  interface RendererResource {
    session: typst.RenderSession;
    renderer: typst.TypstRenderer;
  }

  const [rHandler, setRenderer] = useState<RendererResource | undefined>();
  const [finalArtifact, setArtifact] = useState<Uint8Array | undefined>(artifact);

  useEffect(() => {
    let killPromiseCallback;
    const killPromise = new Promise(resolve => (killPromiseCallback = resolve));

    (async () => {
      /// render after init
      const r = renderer ?? (await createGlobalRenderer(createTypstRenderer, rendererInitOpts));

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

      if (result.diagnostics && onDiagnostics) {
        setDiag(result.diagnostics);
      }

      setArtifact(result.result);
    };

    /// compile after init
    if (compiler) {
      doCompile(compiler);
    } else {
      withGlobalCompiler(createTypstCompiler, compilerInitOpts, doCompile);
    }
  }, [source, compiler]);

  useEffect(() => {
    /// get display layer div
    const divContainerElem = getDisplayLayerDiv();
    if (!divContainerElem) {
      return;
    }
    if (!divContainerElem.firstElementChild) {
      divContainerElem.appendChild(document.createElement('div'));
    }
    const divElem = divContainerElem.firstElementChild as HTMLDivElement;

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
        divContainerElem.style.width = `calc(max(${docWidth * window.devicePixelRatio}px, 100%))`;
      }

      rHandler.renderer.renderToCanvas({
        renderSession: rHandler.session,
        format: 'vector',
        backgroundColor: fill,
        container: divElem,
        pixelPerPt: 3,
      });
    }
  }, [displayDivRef, fill, format, finalArtifact, rHandler]);

  /// --- end: update document --- ///

  return (
    <div>
      {/* todo: remove this embedded css */}
      <style>{htmlLayerCss}</style>
      <div className="typst-app" style={{ height: '0' }} ref={displayDivRef}></div>
    </div>
  );
};
