import { useState, useRef, useEffect } from 'react';
import * as typst from '@myriaddreamin/typst.ts';

export interface TypstDocumentProps {
  fill?: string;
  artifact: string;
}

const renderer = typst.createTypstRenderer((window as unknown as any).pdfjsLib);
const rendererInitReady = renderer.init();

export const TypstDocument = ({ fill, artifact }: TypstDocumentProps) => {
  // const imageContainerRef = useRef(null);

  const [renderer, setRenderer] = useState<typst.TypstRenderer | undefined>();

  useEffect(() => {
    const r = typst.createTypstRenderer((window as unknown as any).pdfjsLib);

    r.init().then(() => {
      setRenderer(r);
    });
  }, []);

  useEffect(() => {
    console.log(fill, artifact.length, renderer);
    //     plugin.render(xhr.responseText, imageContainer).then(renderResult => {
    //       console.log('render done');
    //     });
  }, [fill, artifact, renderer]);

  return (
    <div>
      <div id="imageContainer" style={{ height: '0' }}></div>
      <div>
        <div id="text-layer" className="textLayer"></div>
      </div>
    </div>
  );
};
