import { useEffect, useState } from 'react';
import { TypstDocument } from '../lib';

export const App = () => {
  const [artifact, setArtifact] = useState<Uint8Array>(new Uint8Array(0));

  const getArtifactData = async () => {
    const response = await fetch('http://localhost:20810/skyzh-cv/main.artifact.json').then(response =>
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
