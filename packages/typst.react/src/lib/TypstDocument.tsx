import { useState, useRef, useEffect } from 'react';
import { withGlobalRenderer } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs';
import * as typst from '@myriaddreamin/typst.ts';

// const withGlobalRenderer = (...args: any[]) => {
//   void(args);
//   return undefined;
// };

const htmlLayerCss = `
.typst-html-semantics {
  position: absolute;
  z-index: 2;
  color: transparent;
  font-family: monospace;
  white-space: pre;
}

.typst-html-semantics span {
  color: transparent;
  font-family: monospace;
  transform-origin: left top;
  position: absolute;
  display: inline-block;
  left: 0;
  top: 0;
}

.typst-content-hint {
  position: absolute;
  display: inline-block;
  width: 1px;
  height: 1px;
  overflow: hidden;
}

.typst-html-semantics a {
  position: absolute;
  display: inline-block;
}

/* set transparent itself */
.typst-content-group {
  pointer-events: visible;
}

.typst-html-semantics span::-moz-selection {
  color: transparent;
  background: #7db9dea0;
}

.typst-html-semantics span::selection {
  color: transparent;
  background: #7db9dea0;
}

.typst-html-semantics *::-moz-selection {
  color: transparent;
  background: transparent;
}

.typst-html-semantics *::selection {
  color: transparent;
  background: transparent;
}

.typst-content-fallback {
  color: transparent;
  background: transparent;
}

.pseudo-link,
.typst-text {
  pointer-events: none;
}`;

export interface TypstDocumentProps {
  fill?: string;
  artifact: Uint8Array;
  // todo: add vector format
  format?: 'json';
}

let moduleInitOptions: typst.InitOptions = {
  beforeBuild: [],
  getModule: () => '/node_modules/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
};

export const TypstDocument = ({ fill, artifact, format }: TypstDocumentProps) => {
  /// --- beg: manipulate permission --- ///

  // todo: acquire permission
  const [permission, setPermissionInternal] = useState(false);
  const setPermissionAndOk = (status: PermissionStatus) => {
    if (status.state === 'granted') {
      setPermissionInternal(true);
      return true;
    }
    setPermissionInternal(false);
    return false;
  };
  useEffect(() => {
    navigator.permissions.query({ name: 'local-fonts' as PermissionName }).then(status => {
      if (setPermissionAndOk(status)) {
        return false;
      }
      status.addEventListener('change', event => {
        console.log(event, status);
        setPermissionAndOk(status);
      });
    });
  });

  /// --- end: manipulate permission --- ///

  /// --- beg: update document --- ///
  const displayDivRef = useRef<HTMLDivElement>(null);
  const getDisplayLayerDiv = () => {
    return displayDivRef?.current;
  };

  useEffect(() => {
    const doRender = (renderer: typst.TypstRenderer) => {
      const divElem = getDisplayLayerDiv();
      if (!divElem) {
        return;
      }

      return renderer.render({
        artifactContent: artifact,
        format: 'vector',
        backgroundColor: fill,
        container: divElem,
        pixelPerPt: 3,
      });
    };

    /// get display layer div
    const divElem = getDisplayLayerDiv();
    if (!divElem) {
      return;
    }

    /// we allow empty artifact
    if (!artifact?.length) {
      divElem.innerHTML = '';
      return;
    }

    /// render after init
    withGlobalRenderer(typst.createTypstRenderer, moduleInitOptions, doRender);
  }, [permission, displayDivRef, fill, artifact, format]);

  /// --- end: update document --- ///

  return (
    <div>
      {/* todo: remove this embedded css */}
      <style>{htmlLayerCss}</style>
      <div className="typst-app" style={{ height: '0' }} ref={displayDivRef}></div>
    </div>
  );
};

TypstDocument.setWasmModuleInitOptions = (opts: typst.InitOptions) => {
  moduleInitOptions = opts;
};
