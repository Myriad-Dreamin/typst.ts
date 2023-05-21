import { useState, useRef, useEffect } from 'react';
import { withGlobalRenderer } from '@myriaddreamin/typst.ts/dist/contrib/global-renderer';
import * as typst from '@myriaddreamin/typst.ts';

export interface TypstDocumentProps {
  fill?: string;
  artifact: Uint8Array;
}

// This just queries the existing state of the permission, it does not change it.
async function queryFontPermission() {
  const status = await navigator.permissions.query({ name: 'local-fonts' as PermissionName });
  if (status.state === 'granted') console.log('permission was granted 👍');
  else if (status.state === 'prompt') {
    console.log('permission will be requested');
  } else console.log('permission was denied 👎');
}

export const TypstDocument = ({ fill, artifact }: TypstDocumentProps) => {
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
    if (!permission && displayDivRef?.current?.checkVisibility()) {
      return null;
    }
    return displayDivRef?.current;
  };
  const doRender = (renderer: typst.TypstRenderer) => {
    const divElem = getDisplayLayerDiv();
    if (!divElem) {
      return;
    }
    return renderer.render({
      artifactContent: artifact,
      backgroundColor: fill,
      container: divElem,
      pixelPerPt: 8,
    });
  };

  useEffect(() => {
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
    withGlobalRenderer(
      typst.createTypstRenderer,
      (window as unknown as any).pdfjsLib,
      {
        beforeBuild: [
          typst.preloadRemoteFonts([
            'fonts/LinLibertine_R.ttf',
            'fonts/LinLibertine_RB.ttf',
            'fonts/LinLibertine_RBI.ttf',
            'fonts/LinLibertine_RI.ttf',
            'fonts/NewCMMath-Book.otf',
            'fonts/NewCMMath-Regular.otf',
          ]),
          typst.preloadSystemFonts({
            byFamily: ['Segoe UI Symbol'],
          }),
        ],
        getModule: () =>
          'node_modules/@myriaddreamin/typst-ts-renderer/typst_ts_renderer_bg.wasm',
      },
      doRender,
    );
  }, [permission, displayDivRef, fill, artifact]);

  /// --- end: update document --- ///

  return (
    <div>
      <div className="typst-app" style={{ height: '0' }} ref={displayDivRef}></div>
    </div>
  );
};
