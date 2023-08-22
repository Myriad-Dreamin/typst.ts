import { useEffect, useState } from 'react';
import { TypstDocument } from '../lib';
import * as typst from '@myriaddreamin/typst.ts';

TypstDocument.setWasmModuleInitOptions({
  beforeBuild: [
    typst.preloadRemoteFonts([
      '/fonts/LinLibertine_R.ttf',
      '/fonts/LinLibertine_RB.ttf',
      '/fonts/LinLibertine_RBI.ttf',
      '/fonts/LinLibertine_RI.ttf',
      '/fonts/NewCMMath-Book.otf',
      '/fonts/NewCMMath-Regular.otf',
    ]),
    // typst.preloadSystemFonts({
    //   byFamily: ['Segoe UI Symbol'],
    // }),
  ],
  getModule: () => '/node_modules/@myriaddreamin/typst-ts-renderer/typst_ts_renderer_bg.wasm',
});

export const App = () => {
  const [artifact, setArtifact] = useState<Uint8Array>(new Uint8Array(0));

  const getArtifactData = async () => {
    const response = await fetch('http://localhost:20810/corpus/skyzh-cv/main.white.artifact.json').then(response =>
      response.arrayBuffer(),
    );

    setArtifact(new Uint8Array(response));
  };

  useEffect(() => {
    getArtifactData();
  }, []);

  return (
    <div>
      <h1
        style={{
          color: 'white',
          fontSize: '20px',
          fontFamily: `'Garamond', sans-serif`,
          margin: '20px',
        }}
      >
        Demo: Embed Your Typst Document in React
      </h1>
      <TypstDocument fill="#343541" artifact={artifact} />
    </div>
  );
};
